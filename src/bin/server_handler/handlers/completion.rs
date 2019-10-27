use crate::server_handler::*;

pub struct CompletionRequestHandler;

impl CompletionRequestHandler {
    fn handle_impl(
        context: &mut ServerContext,
        params: CompletionParams,
    ) -> Option<CompletionResponse> {
        let (uri, position) = convert::from_lsp::position_params(params.text_document_position);
        let location = context.server.location(&uri, position)?;
        let completion = context.server.completion(location)?;

        Some(CompletionResponse::List(CompletionList {
            is_incomplete: false,
            items: match completion {
                server::Completion::VariablesInScope(variables) => variables
                    .into_iter()
                    .map(|v| CompletionItem {
                        label: v.name,
                        kind: Some(match v.kind {
                            server::VariableKind::Unknown => CompletionItemKind::Value,
                            server::VariableKind::Class => CompletionItemKind::Class,
                        }),
                        detail: None,
                        documentation: None,
                        deprecated: None,
                        preselect: None,
                        sort_text: None,
                        filter_text: None,
                        insert_text: None,
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        data: None,
                    })
                    .collect(),

                server::Completion::MessageSends(_, signatures) => signatures
                    .into_iter()
                    .map(|s| CompletionItem {
                        label: match s {
                            server::MessageSignature::Unary(s, _) => s,
                            server::MessageSignature::Binary((s, _), _) => s,
                            server::MessageSignature::Keyword(kws, _) => {
                                kws.into_iter().map(|(s, _)| format!("{}:", s)).collect()
                            }
                        },
                        kind: None,
                        detail: None,
                        documentation: None,
                        deprecated: None,
                        preselect: None,
                        sort_text: None,
                        filter_text: None,
                        insert_text: None,
                        insert_text_format: None,
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        data: None,
                    })
                    .collect(),
            },
        }))
    }
}

impl RequestHandler for CompletionRequestHandler {
    type R = request::Completion;

    fn handle(context: &mut ServerContext, params: CompletionParams) -> Option<CompletionResponse> {
        Some(Self::handle_impl(context, params).unwrap_or_else(|| {
            CompletionResponse::List(CompletionList {
                is_incomplete: false,
                items: vec![],
            })
        }))
    }
}
