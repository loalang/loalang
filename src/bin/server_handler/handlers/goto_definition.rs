use crate::server_handler::*;

pub struct GotoDefinitionRequestHandler;
impl RequestHandler for GotoDefinitionRequestHandler {
    type R = request::GotoDefinition;

    fn handle(
        context: &mut ServerContext,
        params: TextDocumentPositionParams,
    ) -> Option<request::GotoDefinitionResponse> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let location = context.server.location(&uri, location)?;
        let usage = context.server.usage(location)?;
        Some(request::GotoDefinitionResponse::Scalar(
            convert::from_loa::span_to_location(usage.declaration.name_span),
        ))
    }
}
