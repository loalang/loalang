use crate::server_handler::*;

pub struct HoverRequestHandler;
impl RequestHandler for HoverRequestHandler {
    type R = request::HoverRequest;

    fn handle(context: &mut ServerContext, params: TextDocumentPositionParams) -> Option<Hover> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let location = context.server.location(&uri, location)?;

        if let Some(expression) = context.server.literal_expression_at(location.clone()) {
            let type_ = context
                .server
                .analysis
                .types
                .get_type_of_expression(&expression);

            return Some(Hover {
                range: Some(convert::from_loa::span_to_range(expression.span)),
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: type_.to_markdown(&context.server.analysis.navigator),
                }),
            });
        }

        let usage = context.server.usage(location.clone())?;

        let markdown = if usage.handle.node.is_message() {
            let behaviour = context.server.behaviour_at(location)?;
            behaviour
                .with_applied_message(
                    &usage.handle.node,
                    &context.server.analysis.navigator,
                    &context.server.analysis.types,
                )
                .to_markdown(&context.server.analysis.navigator)
        } else if usage.declaration.node.is_method() {
            let behaviour = context.server.behaviour_at(location)?;
            behaviour.to_markdown(&context.server.analysis.navigator)
        } else {
            let type_ = context.server.type_at(location);

            if let semantics::Type::Unknown = type_ {
                return None;
            }

            type_.to_markdown(&context.server.analysis.navigator)
        };
        Some(Hover {
            range: Some(convert::from_loa::span_to_range(usage.handle.node.span)),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: markdown,
            }),
        })
    }
}
