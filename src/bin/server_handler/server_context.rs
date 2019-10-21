use crate::server_handler::*;
use serde_json::Value;

pub trait NotificationSender {
    fn send(&self, method: &str, params: Value);
}

pub struct ServerContext<'a> {
    pub server: server::Server,
    notification: &'a dyn NotificationSender,
}

impl<'a> ServerContext<'a> {
    pub fn new(notification: &'a dyn NotificationSender) -> ServerContext<'a> {
        ServerContext {
            server: server::Server::new(),
            notification,
        }
    }

    pub fn publish_updated_diagnostics(&mut self) {
        let diagnostics = self.server.diagnostics();

        for (uri, diagnostics) in diagnostics {
            let params = PublishDiagnosticsParams {
                uri: convert::from_loa::uri_to_url(&uri),
                diagnostics: convert::from_loa::diagnostics_to_diagnostics(diagnostics),
            };

            if let Ok(value) = serde_json::to_value(params) {
                self.notification.send(
                    <notification::PublishDiagnostics as notification::Notification>::METHOD,
                    value,
                );
            }
        }
    }
}
