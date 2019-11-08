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
                .filter_map(|change| match change.range {
                    Some(range) => {
                        let range = convert::from_lsp::range(range);
                        let span = context.server.span(&uri, range)?;
                        Some((span, change.text))
                    }
                    None => Some((
                        Span::all_of(&context.server.module_cells.get(&uri)?.source),
                        change.text,
                    )),
                })
                .collect(),
        );
        context.postpone_publish_updated_diagnostics();
        Some(())
    }
}
