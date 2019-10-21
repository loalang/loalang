#![feature(try_trait)]

extern crate jsonrpc_stdio_server;
extern crate log;
extern crate log_panics;
extern crate lsp_server;
extern crate lsp_types;
extern crate serde_json;
extern crate simple_logging;

use loa::Error;
use lsp_server::*;
use lsp_types::*;
use serde_json::Value;

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

    let sender = NotificationSender { conn: &conn };
    let mut handler =
        server_handler::ServerHandler::new(server_handler::ServerContext::new(&sender));
    let _initialize_params = init(&conn, &server_handler::ServerHandler::CAPABILITIES).unwrap();

    loop {
        match next(&mut handler, &conn) {
            Err(_) => break,
            Ok(()) => (),
        }
    }
}

struct NotificationSender<'a> {
    conn: &'a Connection,
}

impl<'a> server_handler::NotificationSender for NotificationSender<'a> {
    fn send(&self, method: &str, params: Value) {
        let _ = self.conn.sender.send(Message::Notification(Notification {
            method: method.into(),
            params,
        }));
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
                Err(err) => match err {
                    server_handler::ServerError::Empty => result = Some(serde_json::Value::Null),
                    err => {
                        error = Some(ResponseError {
                            code: err.code(),
                            message: err.message(),
                            data: None,
                        })
                    }
                },
            }

            conn.sender
                .send(Message::Response(Response { id, error, result }))?;
        }

        _ => (),
    }

    Ok(())
}
