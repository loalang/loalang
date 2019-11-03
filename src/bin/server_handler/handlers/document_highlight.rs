use crate::server_handler::*;

pub struct DocumentHighlightRequestHandler;
impl RequestHandler for DocumentHighlightRequestHandler {
    type R = request::DocumentHighlightRequest;

    fn handle(
        context: &mut ServerContext,
        params: TextDocumentPositionParams,
    ) -> Option<Vec<DocumentHighlight>> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let location = context.server.location(&uri, location)?;
        let usage = context.server.usage(location)?;
        Some(
            usage
                .named_nodes()
                .into_iter()
                .map(|named_node| named_node.name_span)
                .filter(|span| span.start.uri == uri)
                .map(|span| DocumentHighlight {
                    range: convert::from_loa::span_to_range(span),
                    kind: None,
                })
                .collect(),
        )
    }
}
