use crate::*;
use serde::Deserialize;

extern crate serde_yaml;

#[derive(Deserialize)]
struct FixtureConfig {
    main_class: Option<String>,
    expected: FixtureExpectations,
}

#[derive(Deserialize)]
struct FixtureExpectations {
    success: bool,
    stdout: Vec<String>,
}

#[test]
fn fixtures() {
    let mut unfulfilled_expectations = vec![];
    let mut unexpected_diagnostics = vec![];

    for entry in glob::glob("src/__fixtures__/*").unwrap() {
        let entry = entry.unwrap();

        let mut fixture_config_path = entry.clone();
        fixture_config_path.push("fixture.yml");

        let fixture_config: FixtureConfig =
            serde_yaml::from_reader(std::fs::File::open(fixture_config_path).unwrap()).unwrap();

        let mut source_files_path = entry.clone();
        source_files_path.push("**");
        source_files_path.push("*.loa");

        let mut diagnostics = vec![];
        let mut test_comments = vec![];

        let mut sources = Source::files(source_files_path.to_str().unwrap()).unwrap();
        sources.extend(Source::stdlib().unwrap());
        if let Some(ref main_class) = fixture_config.main_class {
            sources.push(Source::main(main_class));
        }

        let mut analysis: semantics::Analysis = sources
            .into_iter()
            .map(syntax::Parser::new)
            .map(syntax::Parser::parse_with_test_comments)
            .map(|(t, d, c)| {
                diagnostics.extend(d);
                test_comments.extend(c);
                (t.source.uri.clone(), t)
            })
            .into();

        diagnostics.extend(analysis.check());

        let actual_success = !Diagnostic::failed(&diagnostics);

        'expected_comment: for comment in test_comments.iter() {
            for diagnostic in diagnostics.iter() {
                if matches(comment, diagnostic) {
                    continue 'expected_comment;
                }
            }
            unfulfilled_expectations.push(comment.clone());
        }
        'actual_diagnostic: for diagnostic in diagnostics {
            for comment in test_comments.iter() {
                if matches(comment, &diagnostic) {
                    continue 'actual_diagnostic;
                }
            }
            unexpected_diagnostics.push(diagnostic);
        }

        if let Some(_) = fixture_config.main_class {
            let mut generator = generation::Generator::new(&mut analysis);
            let instructions = generator.generate_all().unwrap();

            let mut vm = vm::VM::new();
            let result = vm.eval_pop::<()>(instructions).unwrap();
            let actual_stdout = format!("{}\n", result);
            let expected_stdout: String = fixture_config
                .expected
                .stdout
                .into_iter()
                .map(|s| format!("{}\n", s))
                .collect();

            assert_eq!(actual_stdout, expected_stdout);
        }

        assert_eq!(
            fixture_config.expected.success,
            actual_success,
            "Expected {} to {}",
            entry.to_str().unwrap(),
            if fixture_config.expected.success {
                "be successful"
            } else {
                "fail"
            }
        );
    }

    assert!(
        unexpected_diagnostics.is_empty(),
        "Unexpected diagnostics: {:#?}",
        unexpected_diagnostics
    );
    assert!(
        unfulfilled_expectations.is_empty(),
        "Unfulfilled expectations: {:#?}",
        unfulfilled_expectations
    );
}

fn matches(comment: &syntax::Token, diagnostic: &Diagnostic) -> bool {
    let d_span = diagnostic.span();
    (
        comment.span.start.line,
        comment.span.start.uri.clone(),
        &comment.lexeme()[4..],
    ) == (
        d_span.start.line,
        d_span.start.uri.clone(),
        diagnostic.to_string().as_str(),
    )
}
