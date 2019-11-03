mod server_error;
pub use self::server_error::*;

pub mod convert;

mod request_handler;
pub use self::request_handler::*;

mod notification_handler;
pub use self::notification_handler::*;

mod handlers;
pub use self::handlers::*;

mod server_context;
pub use self::server_context::*;

mod server_handler;
pub use self::server_handler::*;

mod cancellable;
pub use self::cancellable::*;

use loa::*;
use lsp_server::*;
use lsp_types::*;
use serde_json::Value;

pub fn server() {
    let (conn, _threads) = Connection::stdio();
    let conn = Arc::new(conn);

    let sender = Arc::new(NotificationSender { conn: conn.clone() });
    let mut context = ServerContext::new(sender);
    context.server.load_std().expect("failed to load stdlib");
    let initialize_params = init(&conn, &ServerHandler::capabilities()).unwrap();

    conn.sender
        .send(Message::Request(Request::new(
            RequestId::from(loa::Id::new().as_usize() as u64),
            "client/registerCapability".into(),
            RegistrationParams {
                registrations: vec![Registration {
                    id: "workspace-files".into(),
                    method: "workspace/didChangeWatchedFiles".into(),
                    register_options: Some(
                        serde_json::to_value(DidChangeWatchedFilesRegistrationOptions {
                            watchers: vec![FileSystemWatcher {
                                kind: None,
                                glob_pattern: "**/*.loa".into(),
                            }],
                        })
                        .unwrap(),
                    ),
                }],
            },
        )))
        .unwrap();

    if let Some(mut root_path) = initialize_params.root_path.map(std::path::PathBuf::from) {
        root_path.push("**");
        root_path.push("*.loa");

        match root_path.to_str().map(glob::glob) {
            Some(Ok(sources)) => {
                for source in sources.filter_map(|r| r.ok()) {
                    if let Ok(code) = std::fs::read_to_string(&source) {
                        if let Ok(uri) = lsp_types::Url::from_file_path(source)
                            .as_ref()
                            .map(convert::from_lsp::url_to_uri)
                        {
                            context.server.set(uri, code, loa::SourceKind::Module);
                        }
                    }
                }
            }
            _ => (),
        }
    }

    let mut handler = ServerHandler::new(context);

    loop {
        match next(&mut handler, &conn) {
            Err(_) => break,
            Ok(()) => (),
        }
    }
}

pub struct NotificationSender {
    conn: Arc<Connection>,
}

impl NotificationSender {
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

fn next(handler: &mut ServerHandler, conn: &Connection) -> Result<(), Box<dyn Error>> {
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
                    ServerError::Empty => result = Some(serde_json::Value::Null),
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
