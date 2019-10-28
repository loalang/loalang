use crate::server_handler::*;

pub struct HoverRequestHandler;
impl RequestHandler for HoverRequestHandler {
    type R = request::HoverRequest;

    fn handle(
        context: &mut ServerContext,
        params: TextDocumentPositionParams,
    ) -> Option<Hover> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let location = context.server.location(&uri, location)?;
        let usage= context.server.usage(location.clone())?;
        let type_ = context.server.type_at(location);

        if let semantics::Type::Unknown = type_ {
            return None;
        }

        Some(Hover {
            range: Some(convert::from_loa::span_to_range(usage.handle.name_span)),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: type_.to_string(),
            })
        })
    }
}
