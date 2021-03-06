use crate::docs::{BehaviourDoc, ClassDoc};
use crate::server_handler::*;
use loa::semantics::Analysis;
use lsp_types::{Documentation, MarkupContent, MarkupKind};

pub struct CompletionRequestHandler;

impl CompletionRequestHandler {
    fn documentation_of_type(type_: &semantics::Type, analysis: &Analysis) -> String {
        let mut result = type_.to_markdown(&analysis.navigator);

        if let semantics::Type::Class(_, class, _) = type_ {
            if let Some(doc) = analysis
                .navigator
                .find_node(*class)
                .and_then(|c| ClassDoc::extract(analysis, &c))
            {
                result.push_str("\n\n");
                result.push_str(doc.description.to_markdown().as_str());
            }
        }

        result
    }

    fn documentation_of_behaviour(behaviour: &semantics::Behaviour, analysis: &Analysis) -> String {
        let mut result = behaviour.to_markdown(&analysis.navigator);

        if let Some(doc) = BehaviourDoc::extract(analysis, &behaviour) {
            result.push_str("\n\n");
            result.push_str(doc.description.to_markdown().as_str());
        }

        result
    }

    fn handle_impl(
        context: &mut ServerContext,
        params: CompletionParams,
    ) -> Option<CompletionResponse> {
        let (uri, position) = convert::from_lsp::position_params(params.text_document_position);
        let location = context.server.location(&uri, position)?;
        let completion = context.server.completion(location, String::new())?;

        Some(CompletionResponse::List(CompletionList {
            is_incomplete: false,
            items: match completion {
                server::Completion::VariablesInScope(_, variables) => variables
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
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: Self::documentation_of_type(&v.type_, &context.server.analysis),
                        })),
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

                server::Completion::Behaviours(_, behaviours) => behaviours
                    .into_iter()
                    .enumerate()
                    .map(|(i, b)| CompletionItem {
                        label: b.selector(),
                        kind: Some(CompletionItemKind::Method),
                        detail: Some(b.to_string()),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: Self::documentation_of_behaviour(&b, &context.server.analysis),
                        })),
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
