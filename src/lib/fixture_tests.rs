use crate::*;
use serde::Deserialize;

extern crate serde_yaml;
extern crate simple_logging;

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
    simple_logging::log_to_stderr(LevelFilter::Debug);

    let mut failures = vec![];

    for entry in glob::glob("src/__fixtures__/*").unwrap() {
        let entry = entry.unwrap();

        let fixture_name = entry.file_name().and_then(std::ffi::OsStr::to_str).unwrap();
        if fixture_name.starts_with("_") {
            continue;
        }

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

        diagnostics.extend(analysis.check().clone());

        let actual_success = !Diagnostic::failed(&diagnostics);

        'expected_comment: for comment in test_comments.iter() {
            for diagnostic in diagnostics.iter() {
                if matches(comment, diagnostic) {
                    continue 'expected_comment;
                }
            }
            failures.push(format!(
                "Expected diagnostic: {:?} @ {}",
                comment.lexeme(),
                comment.span
            ));
        }
        'actual_diagnostic: for diagnostic in diagnostics {
            for comment in test_comments.iter() {
                if matches(comment, &diagnostic) {
                    continue 'actual_diagnostic;
                }
            }
            failures.push(format!("Unexpected diagnostic: {:#?}", diagnostic));
        }

        if !actual_success {
            continue;
        }

        if let Some(_) = fixture_config.main_class {
            let mut generator = generation::Generator::new(&mut analysis);
            let assembly = generator.generate_all().unwrap();

            let mut vm = vm::VM::new();
            let result = vm.eval_pop::<()>(assembly.clone().into()).unwrap();

            let actual_stdout = format!("{}\n", result);
            let expected_stdout: String = fixture_config
                .expected
                .stdout
                .into_iter()
                .map(|s| format!("{}\n", s))
                .collect();

            if actual_stdout != expected_stdout {
                failures.push(format!(
                    "{}:\nExpected output: {}\n  Actual output: {}",
                    fixture_name, expected_stdout, actual_stdout
                ));
            }
        }

        if fixture_config.expected.success != actual_success {
            failures.push(format!(
                "Expected {} to {}",
                fixture_name,
                if fixture_config.expected.success {
                    "be successful"
                } else {
                    "fail"
                }
            ));
        }
    }

    assert!(failures.is_empty(), "\n\n{}", failures.join("\n\n"));
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
