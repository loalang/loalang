use crate::server_handler::*;

pub struct RenameRequestHandler;

impl RequestHandler for RenameRequestHandler {
    type R = request::Rename;

    fn handle(context: &mut ServerContext, params: RenameParams) -> Option<WorkspaceEdit> {
        let (uri, location) = convert::from_lsp::position_params(params.text_document_position);
        let location = context.server.location(&uri, location)?;
        let usage = context.server.usage(location)?;

        let mut edits = HashMap::new();

        for node in usage.named_nodes() {
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

        let operations = edits
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

        Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(operations)),
        })
    }
}
