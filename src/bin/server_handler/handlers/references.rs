use crate::server_handler::*;

pub struct ReferencesRequestHandler;
impl RequestHandler for ReferencesRequestHandler {
    type R = request::References;

    fn handle(
        context: &mut ServerContext,
        params: ReferenceParams,
    ) -> Option<Vec<lsp_types::Location>> {
        let (uri, location) = convert::from_lsp::position_params(params.text_document_position);
        let location = context.server.location(&uri, location)?;
        let usage = context.server.usage(location)?;
        let mut nodes = usage.references;
        if params.context.include_declaration {
            nodes.insert(0, usage.declaration);
        }
        Some(
            nodes
                .into_iter()
                .map(|n| convert::from_loa::span_to_location(n.name_span))
                .collect(),
        )
    }
}
