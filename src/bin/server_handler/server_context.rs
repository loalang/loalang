use crate::server_handler::*;
use std::sync::atomic::Ordering;

pub struct ServerContext {
    pub server: server::Server,
    notification: Arc<NotificationSender>,
    diagnostics_handle: Option<Cancellable>,
}

impl ServerContext {
    pub fn new(notification: Arc<NotificationSender>) -> ServerContext {
        ServerContext {
            server: server::Server::new(),
            notification,
            diagnostics_handle: None,
        }
    }

    pub fn postpone_publish_updated_diagnostics(&mut self) {
        let handle = std::mem::replace(&mut self.diagnostics_handle, None);
        if let Some(handle) = handle {
            handle.cancel();
        }
        let mut server = self.server.clone();
        let notification = self.notification.clone();

        self.diagnostics_handle = Some(Cancellable::new(move |is_cancelled| {
            let diagnostics = server.diagnostics();

            if is_cancelled.load(Ordering::SeqCst) {
                return;
            }

            for (uri, diagnostics) in diagnostics {
                let params = PublishDiagnosticsParams {
                    uri: convert::from_loa::uri_to_url(&uri),
                    diagnostics: convert::from_loa::diagnostics_to_diagnostics(diagnostics),
                };

                if let Ok(value) = serde_json::to_value(params) {
                    notification.send(
                        <notification::PublishDiagnostics as notification::Notification>::METHOD,
                        value,
                    );
                }
            }
        }))
    }
}
