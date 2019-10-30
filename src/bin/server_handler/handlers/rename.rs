use crate::server_handler::*;
use loa::syntax::{is_valid_binary_selector, is_valid_keyword_selector, is_valid_symbol};

pub struct RenameRequestHandler;

impl RenameRequestHandler {
    fn rename_symbol(
        usage: &server::Usage,
        new_name: String,
    ) -> Option<HashMap<URI, Vec<TextEdit>>> {
        let declared_name = usage.declaration.name.clone();
        let handle_name = usage.handle.name.clone();

        let mut named_nodes = usage.named_nodes();
        if usage.handle_is_aliased() {
            named_nodes.retain(|n| n.name == handle_name);
        } else {
            named_nodes.retain(|n| n.name == declared_name);
        }

        let mut edits = HashMap::new();

        for node in named_nodes {
            Self::add_edit(
                &node.node.span.start.uri,
                &mut edits,
                TextEdit {
                    range: convert::from_loa::span_to_range(node.node.span.clone()),
                    new_text: new_name.clone(),
                },
            );
        }

        Some(edits)
    }

    fn add_edit(uri: &URI, edits: &mut HashMap<URI, Vec<TextEdit>>, edit: TextEdit) {
        if !edits.contains_key(&uri) {
            edits.insert(uri.clone(), vec![]);
        }
        let edits = edits.get_mut(&uri).unwrap();
        edits.push(edit)
    }

    fn rename_behaviour(
        usage: &server::Usage,
        context: &mut ServerContext,
        new_name: String,
    ) -> Option<HashMap<URI, Vec<TextEdit>>> {
        let mut edits = HashMap::new();

        let message_pattern = context
            .server
            .analysis
            .navigator
            .message_pattern_of_method(&usage.declaration.node)?;

        match message_pattern.kind {
            syntax::UnaryMessagePattern { symbol, .. } => {
                let symbol = context
                    .server
                    .analysis
                    .navigator
                    .find_child(&message_pattern, symbol)?;

                if !is_valid_symbol(&new_name) {
                    return None;
                }

                Self::add_edit(
                    &symbol.span.start.uri,
                    &mut edits,
                    TextEdit {
                        range: convert::from_loa::span_to_range(symbol.span.clone()),
                        new_text: new_name.clone(),
                    },
                );
            }

            syntax::BinaryMessagePattern { operator, .. } => {
                let operator = context
                    .server
                    .analysis
                    .navigator
                    .find_child(&message_pattern, operator)?;

                if !is_valid_binary_selector(&new_name) {
                    return None;
                }

                Self::add_edit(
                    &operator.span.start.uri,
                    &mut edits,
                    TextEdit {
                        range: convert::from_loa::span_to_range(operator.span.clone()),
                        new_text: new_name.clone(),
                    },
                );
            }

            syntax::KeywordMessagePattern {
                ref keyword_pairs, ..
            } => {
                if !is_valid_keyword_selector(&new_name, keyword_pairs.len()) {
                    return None;
                }
                let mut new_keywords = new_name.split(":").collect::<Vec<_>>();
                new_keywords.pop();

                for (i, pair) in keyword_pairs.iter().enumerate() {
                    let pair = context
                        .server
                        .analysis
                        .navigator
                        .find_child(&message_pattern, *pair)?;
                    if let syntax::KeywordPair { keyword, .. } = pair.kind {
                        let keyword = context
                            .server
                            .analysis
                            .navigator
                            .find_child(&message_pattern, keyword)?;

                        Self::add_edit(
                            &keyword.span.start.uri,
                            &mut edits,
                            TextEdit {
                                range: convert::from_loa::span_to_range(keyword.span.clone()),
                                new_text: new_keywords[i].to_string(),
                            },
                        );
                    }
                }
            }

            _ => {}
        }

        for reference in usage.references.iter() {
            match reference.node.kind {
                syntax::UnaryMessage { symbol, .. } => {
                    let symbol = context
                        .server
                        .analysis
                        .navigator
                        .find_child(&reference.node, symbol)?;
                    Self::add_edit(
                        &symbol.span.start.uri,
                        &mut edits,
                        TextEdit {
                            range: convert::from_loa::span_to_range(symbol.span.clone()),
                            new_text: new_name.clone(),
                        },
                    );
                }

                syntax::BinaryMessage { operator, .. } => {
                    let operator = context
                        .server
                        .analysis
                        .navigator
                        .find_child(&reference.node, operator)?;
                    Self::add_edit(
                        &operator.span.start.uri,
                        &mut edits,
                        TextEdit {
                            range: convert::from_loa::span_to_range(operator.span.clone()),
                            new_text: new_name.clone(),
                        },
                    );
                }

                syntax::KeywordMessage {
                    ref keyword_pairs, ..
                } => {
                    let mut new_keywords = new_name.split(":").collect::<Vec<_>>();
                    new_keywords.pop();

                    for (i, pair) in keyword_pairs.iter().enumerate() {
                        let pair = context
                            .server
                            .analysis
                            .navigator
                            .find_child(&message_pattern, *pair)?;
                        if let syntax::KeywordPair { keyword, .. } = pair.kind {
                            let keyword = context
                                .server
                                .analysis
                                .navigator
                                .find_child(&message_pattern, keyword)?;

                            Self::add_edit(
                                &keyword.span.start.uri,
                                &mut edits,
                                TextEdit {
                                    range: convert::from_loa::span_to_range(keyword.span.clone()),
                                    new_text: new_keywords[i].to_string(),
                                },
                            );
                        }
                    }
                }

                _ => {}
            }
        }

        Some(edits)
    }
}

impl RequestHandler for RenameRequestHandler {
    type R = request::Rename;

    fn handle(context: &mut ServerContext, params: RenameParams) -> Option<WorkspaceEdit> {
        let (uri, location) = convert::from_lsp::position_params(params.text_document_position);
        let location = context.server.location(&uri, location)?;
        let usage = context.server.usage(location)?;

        let edits = if usage.is_behaviour() {
            Self::rename_behaviour(&usage, context, params.new_name.clone())?
        } else {
            Self::rename_symbol(&usage, params.new_name.clone())?
        };

        let mut operations: Vec<DocumentChangeOperation> = edits
            .into_iter()
            .map(|(uri, edits)| {
                DocumentChangeOperation::Edit(TextDocumentEdit {
                    text_document: VersionedTextDocumentIdentifier {
                        version: None,
                        uri: convert::from_loa::uri_to_url(&uri),
                    },
                    edits,
                })
            })
            .collect();

        if is_renaming_a_declaration_exported_in_file_with_same_name_as_declaration(context, &usage)
        {
            if let Some(new_uri) = usage
                .declaration
                .name_span
                .start
                .uri
                .neighboring_file(format!("{}.loa", params.new_name).as_ref())
            {
                operations.push(DocumentChangeOperation::Op(ResourceOp::Rename(
                    RenameFile {
                        options: None,
                        old_uri: convert::from_loa::uri_to_url(
                            &usage.declaration.name_span.start.uri,
                        ),
                        new_uri: convert::from_loa::uri_to_url(&new_uri),
                    },
                )));
            }
        }

        Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(operations)),
        })
    }
}

fn is_renaming_a_declaration_exported_in_file_with_same_name_as_declaration(
    context: &mut ServerContext,
    usage: &server::Usage,
) -> bool {
    // Make sure we're not renaming the alias
    if usage.handle_is_aliased() {
        return false;
    }

    // Make sure the declaration is exported
    if !context
        .server
        .analysis
        .navigator
        .declaration_is_exported(&usage.declaration.node)
    {
        return false;
    }

    // Make sure the declaration is in a source named the same as the declaration
    if !usage
        .declaration
        .node
        .span
        .start
        .uri
        .matches_basename(format!("{}.loa", usage.declaration.name).as_ref())
    {
        return false;
    }

    true
}
