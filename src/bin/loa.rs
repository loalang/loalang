#![feature(try_trait)]

extern crate jsonrpc_stdio_server;
extern crate log_panics;
extern crate lsp_server;
extern crate lsp_types;
extern crate serde_json;
extern crate simple_logging;

use loa::Error;
use lsp_server::*;
use lsp_types::*;

mod server_handler;

fn main() {
    log_panics::init();
    let log_file = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open("/usr/local/var/log/loa.log")
        .unwrap();
    simple_logging::log_to(log_file, log::LevelFilter::Info);

    let (conn, _threads) = Connection::stdio();

    let mut handler = server_handler::ServerHandler::new();
    let _initialize_params = init(&conn, &handler.capabilities).unwrap();

    loop {
        match next(&mut handler, &conn) {
            Err(_) => break,
            Ok(()) => (),
        }
    }
}

fn init(
    conn: &Connection,
    capabilities: &ServerCapabilities,
) -> Result<InitializeParams, Box<dyn Error>> {
    Ok(serde_json::from_value::<InitializeParams>(
        conn.initialize(serde_json::to_value(capabilities)?)?,
    )?)
}

fn next(
    handler: &mut server_handler::ServerHandler,
    conn: &Connection,
) -> Result<(), Box<dyn Error>> {
    let message = conn.receiver.recv()?;

    match message {
        Message::Notification(Notification { method, params }) => {
            match handler.handle(method.as_ref(), params) {
                _ => (),
            }
        }

        Message::Request(Request { method, params, id }) => {
            let mut error = None;
            let mut result = None;

            match handler.handle(method.as_ref(), params) {
                Ok(r) => result = Some(r),
                Err(server_handler::ServerError(code, s)) => {
                    if code == 0 {
                        result = Some(serde_json::Value::Null)
                    } else {
                        error = Some(ResponseError {
                            code,
                            message: s,
                            data: None,
                        })
                    }
                }
            }

            conn.sender
                .send(Message::Response(Response { id, error, result }))?;
        }

        _ => (),
    }

    Ok(())
}
