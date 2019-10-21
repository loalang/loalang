use crate::server_handler::*;

pub struct DidChangeTextDocumentNotificationHandler;

impl NotificationHandler for DidChangeTextDocumentNotificationHandler {
    type N = notification::DidChangeTextDocument;

    fn handle(context: &mut ServerContext, params: DidChangeTextDocumentParams) -> Option<()> {
        let uri = convert::from_lsp::url_to_uri(&params.text_document.uri);
        context.server.edit(
            params
                .content_changes
                .into_iter()
                .filter_map(|change| {
                    let range = convert::from_lsp::range(change.range?);
                    let span = context.server.span(&uri, range)?;
                    Some((span, change.text))
                })
                .collect(),
        );
        context.publish_updated_diagnostics();
        Some(())
    }
}
