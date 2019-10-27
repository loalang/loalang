use crate::server_handler::*;

pub struct RenameRequestHandler;

impl RequestHandler for RenameRequestHandler {
    type R = request::Rename;

    fn handle(context: &mut ServerContext, params: RenameParams) -> Option<WorkspaceEdit> {
        let (uri, location) = convert::from_lsp::position_params(params.text_document_position);
        let location = context.server.location(&uri, location)?;
        let usage = context.server.usage(location)?;

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
            let uri = node.name_span.start.uri.clone();
            if !edits.contains_key(&uri) {
                edits.insert(uri.clone(), vec![]);
            }
            let edits = edits.get_mut(&uri).unwrap();
            edits.push(TextEdit {
                range: convert::from_loa::span_to_range(node.name_span),
                new_text: params.new_name.clone(),
            })
        }

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
