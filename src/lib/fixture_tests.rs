use crate::vm::Object;
use crate::*;
use serde::Deserialize;
use std::path::PathBuf;
use std::str::FromStr;

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

fn log_to_file() {
    log_panics::init();
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(PathBuf::from_str("/usr/local/var/log/loa.log").unwrap())
        .unwrap();
    simple_logging::log_to(log_file, log::LevelFilter::Info);
}

#[test]
fn fixtures() {
    log_to_file();

    let mut failures = vec![];

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

            let result = match std::panic::catch_unwind::<_, Arc<Object>>(|| {
                let mut vm = vm::VM::new();
                vm.eval_pop::<()>(assembly.clone().into()).unwrap()
            }) {
                Ok(r) => r,
                Err(_) => {
                    eprintln!("{:#?}", assembly);
                    panic!("VM panicked")
                }
            };

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
                    entry.to_str().unwrap(),
                    expected_stdout,
                    actual_stdout
                ));
            }
        }

        if fixture_config.expected.success != actual_success {
            failures.push(format!(
                "Expected {} to {}",
                entry.to_str().unwrap(),
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
