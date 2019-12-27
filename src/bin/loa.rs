#![feature(try_trait, matches_macro)]

extern crate clap;
extern crate colored;
extern crate crypto;
extern crate dirs;
extern crate graphql_client;
extern crate http;
extern crate ignore;
extern crate jsonrpc_stdio_server;
extern crate log;
extern crate log_panics;
extern crate lsp_server;
extern crate lsp_types;
extern crate reqwest;
extern crate rpassword;
extern crate rustyline;
extern crate serde_json;
extern crate serde_yaml;
extern crate simple_logging;
extern crate tar;
extern crate tee;

mod docs;
mod repl;
mod reporting;
mod server_handler;
pub use self::reporting::*;
mod runtime;
use self::runtime::ServerRuntime;

mod pkg;

fn log_to_file() {
    log_panics::init();
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open({
            let mut log_file = dirs::config_dir().unwrap();
            log_file.push("loa");
            std::fs::create_dir_all(&log_file).expect("need write permission to config directory");
            log_file.push("loa.log");
            log_file
        })
        .unwrap();
    simple_logging::log_to(log_file, log::LevelFilter::Info);
}

fn log_to_stderr() {
    #[cfg(debug_assertions)]
    simple_logging::log_to_stderr(LevelFilter::Debug);
    #[cfg(not(debug_assertions))]
    simple_logging::log_to_stderr(LevelFilter::Error);
}

fn main() -> Result<(), clap::Error> {
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

    let mut config_file = dirs::config_dir().unwrap();
    config_file.push("loa");
    std::fs::create_dir_all(&config_file).expect("need write permission to config directory");
    config_file.push("loapkg.json");

    let mut app = clap::App::new("loa")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommands(vec![
            clap::SubCommand::with_name("server"),
            clap::SubCommand::with_name("repl"),
            clap::SubCommand::with_name("build")
                .arg(
                    clap::Arg::with_name("out")
                        .long("out")
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
            clap::SubCommand::with_name("format").arg(
                clap::Arg::with_name("files")
                    .takes_value(true)
                    .multiple(true)
                    .value_name("FILES"),
            ),
            clap::SubCommand::with_name("docs").subcommands(vec![
                clap::SubCommand::with_name("inspect")
                    .arg(
                        clap::Arg::with_name("all")
                            .help("Include stdlib and dependencies")
                            .long("all")
                            .short("a"),
                    )
                    .arg(
                        clap::Arg::with_name("format")
                            .help("Output format (json|yaml)")
                            .long("format")
                            .short("f")
                            .takes_value(true)
                            .value_name("FORMAT")
                            .default_value("json"),
                    ),
                clap::SubCommand::with_name("serve").arg(
                    clap::Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .takes_value(true)
                        .value_name("PORT")
                        .default_value("7065"),
                ),
            ]),
            clap::SubCommand::with_name("pkg")
                .arg(
                    clap::Arg::with_name("server")
                        .long("server")
                        .short("s")
                        .takes_value(true)
                        .value_name("SERVER_HOST")
                        .default_value("https://api.loalang.xyz"),
                )
                .arg(
                    clap::Arg::with_name("config")
                        .long("config")
                        .short("c")
                        .takes_value(true)
                        .value_name("CONFIG_FILE")
                        .default_value(config_file.to_str().unwrap()),
                )
                .subcommands(vec![
                    clap::SubCommand::with_name("login"),
                    clap::SubCommand::with_name("logout"),
                    clap::SubCommand::with_name("whoami"),
                    clap::SubCommand::with_name("get").arg(
                        clap::Arg::with_name("no-update")
                            .long("no-update")
                            .short("n"),
                    ),
                    clap::SubCommand::with_name("add").arg(
                        clap::Arg::with_name("package")
                            .takes_value(true)
                            .multiple(true)
                            .value_name("PACKAGE_NAME"),
                    ),
                    clap::SubCommand::with_name("remove").arg(
                        clap::Arg::with_name("package")
                            .takes_value(true)
                            .multiple(true)
                            .value_name("PACKAGE_NAME"),
                    ),
                    clap::SubCommand::with_name("publish").arg(
                        clap::Arg::with_name("version")
                            .takes_value(true)
                            .value_name("VERSION"),
                    ),
                ]),
        ]);
    let cli = app.clone().get_matches();

    if let None = cli.subcommand_name() {
        app.print_help()?;
        println!();
        return Ok(());
    }

    match cli.subcommand() {
        ("repl", _) => {
            log_to_file();
            repl::repl()
        }

        ("server", _) => {
            log_to_file();
            server_handler::server()
        }

        ("format", Some(matches)) => {
            log_to_file();
            matches
                .values_of("files")
                .map(|f| f.collect())
                .unwrap_or(vec!["**/*.loa"])
                .into_iter()
                .map(glob::glob)
                .filter_map(Result::ok)
                .flat_map(identity)
                .filter_map(Result::ok)
                .map(loa::Source::file)
                .filter_map(Result::ok)
                .map(loa::syntax::Parser::new)
                .map(loa::syntax::Parser::parse)
                .for_each(|(tree, _)| println!("{:#}", tree));
        }

        ("exec", Some(matches)) => match matches.value_of("loabin") {
            None => {
                app.print_help()?;
                println!();
            }
            Some(file) => {
                log_to_stderr();
                let instructions = std::fs::read(file)
                    .map(|bytes| Instructions::from_bytes(bytes.as_slice()).unwrap())?;

                let mut vm = VM::new();
                if let Some(result) = vm.eval_pop::<ServerRuntime>(instructions) {
                    println!("{}", result);
                }
            }
        },

        ("run", Some(matches)) => {
            log_to_stderr();
            let instructions = build(matches.value_of("main").unwrap());

            if let Some(result) = loa::vm::VM::new().eval_pop::<ServerRuntime>(instructions) {
                println!("{}", result);
            }
        }

        ("build", Some(matches)) => {
            log_to_stderr();
            let instructions = build(matches.value_of("main").unwrap());

            match matches.value_of("out") {
                None => {
                    stdout()
                        .write(instructions.to_bytes().unwrap().as_slice())
                        .unwrap();
                }
                Some(outfile) => {
                    std::fs::write(outfile, instructions.to_bytes().unwrap()).unwrap();
                }
            }
        }

        ("docs", Some(matches)) => match matches.subcommand() {
            ("serve", Some(matches)) => {
                log_to_stderr();
                let port_str = matches.value_of("port").unwrap();
                match u16::from_str(port_str) {
                    Ok(port) => {
                        let analysis = parse_and_report(None);
                        docs::serve(port, analysis.into())
                    }
                    Err(_) => eprintln!("Invalid port: {}", port_str),
                }
            }
            ("inspect", Some(matches)) => {
                log_to_file();
                let analysis = parse_and_report(None);
                let lockfile = ManifestFile::new(".pkg.lock");
                let pkgfile = ManifestFile::new("pkg.yml");
                let mut docs: Docs = analysis.into();
                if let (Ok(pkgfile), Ok(lockfile)) = (pkgfile.load(), lockfile.load()) {
                    if !matches.is_present("all") {
                        docs.retain_package(project_name.as_ref());
                    }
                    docs.apply_versions(&Versions { pkgfile, lockfile });
                }
                match matches.value_of("format").unwrap() {
                    "json" => {
                        println!("{}", serde_json::to_string_pretty(&docs).unwrap());
                    }
                    "yaml" => {
                        println!("{}", serde_yaml::to_string(&docs).unwrap());
                    }
                    f => eprintln!(
                        "Invalid format {}. Please choose between json and yaml.\n{}",
                        f,
                        matches.usage()
                    ),
                }
            }
            _ => eprintln!("{}", matches.usage()),
        },

        ("pkg", Some(matches)) => {
            let (_, analysis) = parse(None);

            let api = pkg::APIClient::new(
                matches.value_of("server").unwrap(),
                matches.value_of("config").unwrap(),
                analysis,
            );

            match matches.subcommand() {
                ("login", _) => {
                    let mut editor = rustyline::Editor::<()>::new();
                    if let Ok(email) = editor.readline("Email: ") {
                        if let Ok(password) = rpassword::read_password_from_tty(Some("Password: "))
                        {
                            if let Err(e) = api.login(email.as_ref(), password.as_ref()) {
                                eprintln!("{}", e);
                            } else {
                                println!("Successfully logged in as {}.", email);
                            }
                        }
                    }
                }
                ("logout", _) => {
                    if let Err(e) = api.logout() {
                        eprintln!("{}", e)
                    }
                }
                ("get", Some(matches)) => {
                    if let Err(e) = {
                        if matches.is_present("no-update") {
                            api.get_from_lockfile()
                        } else {
                            api.get_from_pkgfile()
                        }
                    } {
                        eprintln!("{}", e)
                    }
                }
                ("whoami", _) => match api.auth_email() {
                    Err(e) => eprintln!("{}", e),
                    Ok(Some(email)) => println!("{}", email),
                    Ok(None) => println!("Not logged in"),
                },
                ("add", Some(matches)) => match matches.values_of("package") {
                    Some(packages) => {
                        if let Err(e) = api.add_packages(packages.map(|p| (p, None)).collect()) {
                            eprintln!("{}", e);
                        }
                    }
                    None => eprintln!("{}", matches.usage()),
                },
                ("remove", Some(matches)) => match matches.values_of("package") {
                    Some(packages) => {
                        if let Err(e) = api.remove_packages(packages.collect()) {
                            eprintln!("{}", e);
                        }
                    }
                    None => eprintln!("{}", matches.usage()),
                },
                ("publish", Some(matches)) => match matches.value_of("version") {
                    Some(version) => {
                        if let Err(e) = api.publish_package(project_name.as_ref(), version) {
                            eprintln!("{}", e);
                        } else {
                            println!(
                                "Successfully published {} version {}!",
                                project_name, version
                            );
                        }
                    }
                    None => eprintln!("{}", matches.usage()),
                },
                _ => eprintln!("{}", matches.usage()),
            }
        }

        _ => eprintln!("{}", cli.usage()),
    }

    Ok(())
}

use crate::docs::{Docs, Versions};
use crate::pkg::ManifestFile;
use loa::generation::Instructions;
use loa::vm::VM;
use log::LevelFilter;
use std::convert::identity;
use std::io::{stdout, Write};
use std::process::exit;
use std::str::FromStr;

fn parse(main: Option<&str>) -> (Vec<loa::Diagnostic>, loa::semantics::Analysis) {
    let mut sources = loa::Source::stdlib().expect("failed to load stdlib");
    sources.extend(loa::Source::files("**/*.loa").expect("failed to read in sources"));

    if let Some(main) = main {
        sources.push(loa::Source::main(main));
    }

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

    (diagnostics, analysis)
}

fn parse_and_report(main: Option<&str>) -> loa::semantics::Analysis {
    let (diagnostics, analysis) = parse(main);

    if loa::Diagnostic::failed(&diagnostics) {
        <PrettyReporter as loa::Reporter>::report(diagnostics, &analysis.navigator);
        exit(1);
    }
    <PrettyReporter as loa::Reporter>::report(diagnostics, &analysis.navigator);

    analysis
}

fn build(main: &str) -> Instructions {
    let mut analysis = parse_and_report(Some(main));

    let mut generator = loa::generation::Generator::new(&mut analysis);
    match generator.generate_all() {
        Err(err) => {
            eprintln!("{:?}", err);
            exit(1);
        }
        Ok(i) => i,
    }
}
