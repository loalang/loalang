#![feature(try_trait)]

extern crate clap;
extern crate colored;
extern crate jsonrpc_stdio_server;
extern crate log;
extern crate log_panics;
extern crate lsp_server;
extern crate lsp_types;
extern crate rustyline;
extern crate serde_json;
extern crate simple_logging;

mod repl;
mod reporting;
mod server_handler;
pub use self::reporting::*;

fn main() -> Result<(), clap::Error> {
    log_panics::init();
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/usr/local/var/log/loa.log")
        .unwrap();
    simple_logging::log_to(log_file, log::LevelFilter::Info);

    let project_name = std::env::current_dir()
        .ok()
        .as_ref()
        .and_then(|b| b.file_name())
        .and_then(|s| s.to_str())
        .map(String::from)
        .unwrap_or("App".into());

    let default_main = format!("{}/Main", project_name);
    let default_out = format!("{}.loabin", project_name);

    let arg = clap::Arg::with_name("main")
        .takes_value(true)
        .default_value(default_main.as_ref())
        .value_name("MAIN_CLASS");

    let mut app = clap::App::new("loa").subcommands(vec![
        clap::SubCommand::with_name("server"),
        clap::SubCommand::with_name("repl"),
        clap::SubCommand::with_name("build")
            .arg(
                clap::Arg::with_name("out")
                    .short("o")
                    .takes_value(true)
                    .default_value(default_out.as_ref()),
            )
            .arg(arg.clone()),
        clap::SubCommand::with_name("run").arg(arg),
        clap::SubCommand::with_name("exec").arg(
            clap::Arg::with_name("loabin")
                .takes_value(true)
                .value_name("BINARY_FILE"),
        ),
    ]);
    let cli = app.clone().get_matches();

    if let None = cli.subcommand_name() {
        app.print_help()?;
        println!();
        return Ok(());
    }

    match cli.subcommand() {
        ("repl", _) => repl::repl(),

        ("server", _) => server_handler::server(),

        ("exec", Some(matches)) => match matches.value_of("loabin") {
            None => app.print_help()?,
            Some(file) => {
                let instructions = std::fs::read(file)
                    .map(|bytes| Instructions::from_bytes(bytes.as_slice()).unwrap())?;

                let mut vm = VM::new();
                if let Some(result) = vm.eval(&instructions) {
                    println!("{}", result);
                }
            }
        },

        ("run", Some(matches)) => match build(matches.value_of("main").unwrap()) {
            Err(err) => eprintln!("ERROR during generation: {:?}", err),
            Ok(instructions) => {
                if let Some(result) = loa::vm::VM::new().eval(&instructions) {
                    println!("{}", result);
                }
            }
        },

        ("build", Some(matches)) => match build(matches.value_of("main").unwrap()) {
            Err(err) => eprintln!("ERROR during generation: {:?}", err),
            Ok(instructions) => match matches.value_of("out") {
                None => {
                    stdout()
                        .write(instructions.to_bytes().unwrap().as_slice())
                        .unwrap();
                }
                Some(outfile) => {
                    std::fs::write(outfile, instructions.to_bytes().unwrap()).unwrap();
                }
            },
        },

        _ => eprintln!("{}", cli.usage()),
    }

    Ok(())
}

use loa::generation::{GenerationResult, Instructions};
use loa::vm::VM;
use std::io::{stdout, Write};

fn build(main: &str) -> GenerationResult {
    let mut sources = loa::Source::files("**/*.loa").expect("failed to read in sources");

    sources.push(loa::Source::new(
        loa::SourceKind::REPLLine,
        loa::URI::REPLLine(0),
        format!("import {} as Main. Main run.", main),
    ));

    let mut diagnostics = vec![];
    let modules = sources
        .iter()
        .cloned()
        .map(loa::syntax::Parser::new)
        .map(loa::syntax::Parser::parse)
        .map(|(tree, d)| {
            diagnostics.extend(d);
            (tree.source.uri.clone(), tree)
        })
        .collect();
    let mut analysis = loa::semantics::Analysis::new(loa::Arc::new(modules));
    diagnostics.extend(analysis.check());

    if loa::Diagnostic::report::<PrettyReporter>(diagnostics, &analysis.navigator) {
        return Ok(vec![].into());
    }

    let mut generator = loa::generation::Generator::new(&mut analysis);
    let mut instructions = Instructions::new();
    for source in sources {
        instructions.extend(generator.generate(&source.uri)?);
    }
    Ok(instructions)
}
