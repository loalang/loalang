use crate::server_handler::*;

pub struct PrepareRenameRequestHandler;

impl RequestHandler for PrepareRenameRequestHandler {
    type R = request::PrepareRenameRequest;

    fn handle(
        context: &mut ServerContext,
        params: TextDocumentPositionParams,
    ) -> Option<PrepareRenameResponse> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let location = context.server.location(&uri, location)?;
        let usage = context.server.usage(location)?;
        Some(PrepareRenameResponse::RangeWithPlaceholder {
            range: convert::from_loa::span_to_range(usage.declaration.name_span),
            placeholder: usage.declaration.name,
        })
    }
}
