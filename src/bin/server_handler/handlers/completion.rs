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
                    .enumerate()
                    .map(|(i, v)| CompletionItem {
                        label: v.name,
                        kind: Some(match v.kind {
                            server::VariableKind::Unknown => CompletionItemKind::Value,
                            server::VariableKind::Class => CompletionItemKind::Class,
                            server::VariableKind::Parameter => CompletionItemKind::Variable,
                        }),
                        detail: Some(v.type_.to_string()),
                        documentation: None,
                        deprecated: None,
                        preselect: Some(i == 0),
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

                server::Completion::Behaviours(behaviours) => behaviours
                    .into_iter()
                    .enumerate()
                    .map(|(i, b)| CompletionItem {
                        label: b.selector(),
                        kind: Some(CompletionItemKind::Method),
                        detail: Some(b.to_string()),
                        documentation: None,
                        deprecated: None,
                        preselect: Some(i == 0),
                        sort_text: None,
                        filter_text: None,
                        insert_text: Some(match b.message {
                            semantics::BehaviourMessage::Unary(ref s) => s.clone(),
                            semantics::BehaviourMessage::Binary(ref s, _) => format!("{} $1", s),
                            semantics::BehaviourMessage::Keyword(ref kws) => kws
                                .iter()
                                .enumerate()
                                .map(|(i, (s, _))| format!("{}: ${}", s, i + 1))
                                .collect::<Vec<_>>()
                                .join(" "),
                        }),
                        insert_text_format: Some(InsertTextFormat::Snippet),
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
