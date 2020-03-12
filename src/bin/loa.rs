#![feature(try_trait)]

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
use colored::Colorize;

mod pkg;

use crate::docs::{Docs, Versions};
use crate::pkg::ManifestFile;
use loa::bytecode::BytecodeEncoding;
use loa::bytecode::Instruction;
use loa::optimization::Optimizable;
use loa::vm::VM;
use log::LevelFilter;
use std::convert::identity;
use std::io::stdout;
use std::process::exit;
use std::str::FromStr;

fn log_to_file() {
    log_panics::init();
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(loa::sdk_path(&["log", "loa.log"]))
        .unwrap();
    #[cfg(debug_assertions)]
    simple_logging::log_to(log_file, LevelFilter::Info);
    #[cfg(not(debug_assertions))]
    simple_logging::log_to(log_file, LevelFilter::Error);
}

fn log_to_stderr() {
    #[cfg(debug_assertions)]
    simple_logging::log_to_stderr(LevelFilter::Info);
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

    let main_class_option = clap::Arg::with_name("main")
        .help("The qualified identifier of the class to use as the program's entrypoint.")
        .takes_value(true)
        .default_value(default_main.as_ref())
        .value_name("MAIN_CLASS");

    let no_stdlib_option = clap::Arg::with_name("no_stdlib")
        .long("no-stdlib")
        .help("Don't include the standard library.");

    let mut config_file = dirs::config_dir().unwrap();
    config_file.push("loa");
    std::fs::create_dir_all(&config_file).expect("need write permission to config directory");
    config_file.push("loapkg.json");

    let mut app = clap::App::new("loa")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Compiler Toolchain for the Loa Programming Language. Visit https://loalang.xyz for more information.")
        .subcommands(vec![
            clap::SubCommand::with_name("server")
                .about("Starts a Language Server using STDIO. Used by editors to provide an integrated experience for Loa development."),
            clap::SubCommand::with_name("repl")
                .about("Starts an interactive Read-Eval-Print-Loop that can be used to quickly explore APIs and make quick calculations.")
                .arg(no_stdlib_option.clone()),
            clap::SubCommand::with_name("build")
                .about("Builds the current project into a Loa VM bytecode file, that can be executed in many different environments using the Loa VM.")
                .arg(
                    clap::Arg::with_name("out")
                        .help("The output file path for the bytecode file.")
                        .long("out")
                        .short("o")
                        .takes_value(true)
                        .default_value(default_out.as_ref()),
                )
                .arg(
                    clap::Arg::with_name("output_assembly")
                        .help("Output Loa VM Assembly.")
                        .long("assembly")
                        .short("s"),
                )
                .arg(no_stdlib_option.clone())
                .arg(main_class_option.clone()),
            clap::SubCommand::with_name("run")
                .about("Builds and immediately runs the current project. This is not suitable for a production environment, but handy for quickly running your program.")
                .arg(no_stdlib_option)
                .arg(main_class_option),
            clap::SubCommand::with_name("exec")
                .about("Executes a bytecode file using the Loa VM.")
                .arg(
                clap::Arg::with_name("loabin")
                    .help("The path to the bytecode file (.loabin).")
                    .takes_value(true)
                    .value_name("BINARY_FILE"),
            ),
            clap::SubCommand::with_name("format")
                .about("Runs the Loa code formatter on the provided files. Outputs to stdout, and does not modify the files themselves.")
                .arg(
                clap::Arg::with_name("files")
                    .help("The files that will be formatted.")
                    .takes_value(true)
                    .multiple(true)
                    .value_name("FILES"),
            ),
            clap::SubCommand::with_name("docs")
                .about("Commands regarding automatically generated API documentation for the current project.")
                .subcommands(vec![
                clap::SubCommand::with_name("inspect")
                    .about("Output generated docs to stdout.")
                    .arg(
                        clap::Arg::with_name("all")
                            .help("Include stdlib and dependencies.")
                            .long("all")
                            .short("a"),
                    )
                    .arg(
                        clap::Arg::with_name("format")
                            .help("Output format (json|yaml).")
                            .long("format")
                            .short("f")
                            .takes_value(true)
                            .value_name("FORMAT")
                            .default_value("json"),
                    ),
                clap::SubCommand::with_name("serve")
                    .about("Starts a local web server, with a user friendly interface for browsing documentation.")
                    .arg(
                    clap::Arg::with_name("port")
                        .help("The local port to start the server on.")
                        .long("port")
                        .short("p")
                        .takes_value(true)
                        .value_name("PORT")
                        .default_value("7065"),
                ),
            ]),
            clap::SubCommand::with_name("pkg")
                .about("Commands regarding the Loa Package Manager.")
                .arg(
                    clap::Arg::with_name("server")
                        .help("The remote server to use for resolving and downloading dependencies.")
                        .long("server")
                        .short("s")
                        .takes_value(true)
                        .value_name("SERVER_HOST")
                        .default_value("https://api.loalang.xyz"),
                )
                .arg(
                    clap::Arg::with_name("config")
                        .help("The config file to use for storing authentication details etc.")
                        .long("config")
                        .short("c")
                        .takes_value(true)
                        .value_name("CONFIG_FILE")
                        .default_value(config_file.to_str().unwrap()),
                )
                .subcommands(vec![
                    clap::SubCommand::with_name("login")
                        .about("Authenticate yourself with the remote server."),
                    clap::SubCommand::with_name("logout")
                        .about("Remove any authentication previously established with the server."),
                    clap::SubCommand::with_name("whoami")
                        .about("Display details about the currently logged in user."),
                    clap::SubCommand::with_name("get")
                        .about("Download and install any registered dependencies.").arg(
                        clap::Arg::with_name("no-update")
                            .help("Only install exactly what's in the lockfile, as opposed to resolving appropriate versions and regenerating the lockfile.")
                            .long("no-update")
                            .short("n"),
                    ),
                    clap::SubCommand::with_name("add")
                        .about("Add, download, and install a new dependency of this project.")
                        .arg(
                        clap::Arg::with_name("package")
                            .help("The name of the package to add.")
                            .takes_value(true)
                            .multiple(true)
                            .value_name("PACKAGE_NAME"),
                    ),
                    clap::SubCommand::with_name("remove")
                        .about("Remove and uninstall an existing dependency of this project.").arg(
                        clap::Arg::with_name("package")
                            .help("The name of the package to remove.")
                            .takes_value(true)
                            .multiple(true)
                            .value_name("PACKAGE_NAME"),
                    ),
                    clap::SubCommand::with_name("publish")
                        .about("Pack and upload a version of the current project.")
                        .arg(
                        clap::Arg::with_name("version")
                            .help("The version number of this release.")
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
        ("repl", Some(matches)) => {
            log_to_stderr();
            repl::repl(!matches.is_present("no_stdlib"))
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
                let instructions = if file.ends_with(".loaasm") {
                    let assembly_code = std::fs::read_to_string(file).unwrap();
                    let assembly = loa::assembly::Parser::new()
                        .parse(assembly_code.as_ref())
                        .unwrap();
                    assembly.into()
                } else {
                    std::fs::read(file)
                        .map(|bytes| Vec::<_>::deserialize(bytes.as_slice()).unwrap())?
                };

                let mut vm = VM::new();
                if let Some(result) = vm.eval_pop::<ServerRuntime>(instructions) {
                    println!("{}", result);
                }
            }
        },

        ("run", Some(matches)) => {
            log_to_stderr();
            let assembly = build(
                matches.value_of("main").unwrap(),
                matches.is_present("no_stdlib"),
            );

            if let Some(result) = loa::vm::VM::new().eval_pop::<ServerRuntime>(assembly.into()) {
                println!("{}", result);
            }
        }

        ("build", Some(matches)) => {
            log_to_stderr();
            let output_assembly = matches.is_present("output_assembly");
            let mut assembly = build(
                matches.value_of("main").unwrap(),
                matches.is_present("no_stdlib"),
            );

            assembly.optimize();

            let mut write: Box<dyn std::io::Write> = match matches.value_of("out") {
                None | Some("") => Box::new(stdout()),
                Some(outfile) => {
                    let mut outfile = outfile.to_string();

                    if output_assembly && outfile == default_out {
                        outfile.pop();
                        outfile.pop();
                        outfile.pop();
                        outfile.push_str("asm");
                    }

                    println!("{} {}", "Building".bright_black(), outfile.green());

                    let outfile_sink = std::fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .open(outfile)
                        .unwrap();

                    Box::new(outfile_sink)
                }
            };

            if output_assembly {
                write.write(format!("{:?}", assembly).as_bytes())?;
            } else {
                let instructions: Vec<Instruction> = assembly.into();
                instructions.serialize(write)?;
            }
        }

        ("docs", Some(matches)) => match matches.subcommand() {
            ("serve", Some(matches)) => {
                log_to_stderr();
                let port_str = matches.value_of("port").unwrap();
                match u16::from_str(port_str) {
                    Ok(port) => {
                        let analysis = parse_and_report(None, true);
                        docs::serve(port, analysis.into())
                    }
                    Err(_) => eprintln!("Invalid port: {}", port_str),
                }
            }
            ("inspect", Some(matches)) => {
                log_to_file();
                let analysis = parse_and_report(None, true);
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
            let (_, analysis) = parse(None, true);

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

fn parse(
    main: Option<&str>,
    load_stdlib: bool,
) -> (Vec<loa::Diagnostic>, loa::semantics::Analysis) {
    let mut sources = if load_stdlib {
        vec![]
    } else {
        loa::Source::stdlib().expect("failed to load stdlib")
    };
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
    diagnostics.extend(analysis.check().clone());

    (diagnostics, analysis)
}

fn parse_and_report(main: Option<&str>, load_stdlib: bool) -> loa::semantics::Analysis {
    let (diagnostics, analysis) = parse(main, load_stdlib);

    if loa::Diagnostic::failed(&diagnostics) {
        <PrettyReporter as loa::Reporter>::report(diagnostics, &analysis.navigator);
        exit(1);
    }
    <PrettyReporter as loa::Reporter>::report(diagnostics, &analysis.navigator);

    analysis
}

fn build(main: &str, load_stdlib: bool) -> loa::assembly::Assembly {
    let mut analysis = parse_and_report(Some(main), load_stdlib);

    let mut generator = loa::generation::Generator::new(&mut analysis);
    match generator.generate_all() {
        Err(err) => {
            eprintln!("{:?}", err);
            exit(1);
        }
        Ok(i) => i,
    }
}
