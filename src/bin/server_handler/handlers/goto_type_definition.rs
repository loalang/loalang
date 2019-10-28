use crate::server_handler::*;
use loa::semantics::Navigator;

pub struct GotoTypeDefinitionRequestHandler;
impl RequestHandler for GotoTypeDefinitionRequestHandler {
    type R = request::GotoTypeDefinition;

    fn handle(
        context: &mut ServerContext,
        params: TextDocumentPositionParams,
    ) -> Option<request::GotoTypeDefinitionResponse> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let location = context.server.location(&uri, location)?;
        let type_ = context.server.type_at(location);

        match type_ {
            semantics::Type::Unknown => None,
            semantics::Type::Class(_, id, _) | semantics::Type::Parameter(_, id, _) => {
                let navigator = context.server.navigator();
                let declaration = navigator.find_node(id)?;
                let (_, s) = navigator.symbol_of(&declaration)?;
                Some(convert::from_loa::span_to_location(s.span).into())
            }
        }
    }
}
