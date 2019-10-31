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
mod server_handler;

fn main() -> Result<(), clap::Error> {
    log_panics::init();
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/usr/local/var/log/loa.log")
        .unwrap();
    simple_logging::log_to(log_file, log::LevelFilter::Info);

    let mut app = clap::App::new("loa").subcommands(vec![
        clap::SubCommand::with_name("server"),
        clap::SubCommand::with_name("repl"),
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

        _ => eprintln!("{}", cli.usage()),
    }

    Ok(())
}
