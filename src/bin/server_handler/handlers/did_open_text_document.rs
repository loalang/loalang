use crate::server_handler::*;

pub struct DidOpenTextDocumentNotificationHandler;

impl NotificationHandler for DidOpenTextDocumentNotificationHandler {
    type N = notification::DidOpenTextDocument;

    fn handle(context: &mut ServerContext, params: DidOpenTextDocumentParams) -> Option<()> {
        let uri = convert::from_lsp::url_to_uri(&params.text_document.uri);
        context
            .server
            .set(uri, params.text_document.text, loa::SourceKind::Module);
        context.postpone_publish_updated_diagnostics();
        Some(())
    }
}
