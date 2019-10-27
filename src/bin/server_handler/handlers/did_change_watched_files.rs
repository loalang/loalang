use crate::server_handler::*;

pub struct DidChangeWatchedFilesNotificationHandler;

impl NotificationHandler for DidChangeWatchedFilesNotificationHandler {
    type N = notification::DidChangeWatchedFiles;

    fn handle(context: &mut ServerContext, params: DidChangeWatchedFilesParams) -> Option<()> {
        for change in params.changes {
            let uri = convert::from_lsp::url_to_uri(&change.uri);
            use lsp_types::FileChangeType::*;
            match change.typ {
                Changed => (),
                Created => (),
                Deleted => context.server.remove(uri),
            }
        }
        Some(())
    }
}
