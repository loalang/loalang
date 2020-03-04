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

        let placeholder = if usage.is_method() {
            let message_pattern = context
                .server
                .analysis
                .navigator
                .message_pattern_of_method(&usage.declaration.node)?;
            let selector = context
                .server
                .analysis
                .navigator
                .message_pattern_selector(&message_pattern)?;

            selector
        } else if usage.is_initializer() {
            let message_pattern = context
                .server
                .analysis
                .navigator
                .message_pattern_of_initializer(&usage.declaration.node)?;
            let selector = context
                .server
                .analysis
                .navigator
                .message_pattern_selector(&message_pattern)?;

            selector
        } else {
            usage.handle.name
        };

        Some(PrepareRenameResponse::RangeWithPlaceholder {
            range: convert::from_loa::span_to_range(usage.handle.name_span),
            placeholder,
        })
    }
}
