use crate::server_handler::ServerContext;
use lsp_types::notification::Notification;

pub trait NotificationHandler {
    type N: Notification;

    fn handle(context: &mut ServerContext, params: <Self::N as Notification>::Params)
        -> Option<()>;
}
