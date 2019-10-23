use crate::server_handler::*;

pub struct CodeActionRequestHandler;

impl RequestHandler for CodeActionRequestHandler {
    type R = request::CodeActionRequest;

    fn handle(
        context: &mut ServerContext,
        params: CodeActionParams,
    ) -> Option<Vec<CodeActionOrCommand>> {
        let url = params.text_document.uri.clone();
        let uri = convert::from_lsp::url_to_uri(&params.text_document.uri);
        let span = context
            .server
            .span(&uri, convert::from_lsp::range(params.range))?;
        let source = context.server.source(&uri)?;
        let tree = context.server.tree(&uri)?;
        let namespace = tree.namespace();
        let end_of_import_list = tree.end_of_import_list_location();

        let mut actions = vec![];

        for diagnostic in params.context.diagnostics {
            let loa_diagnostic =
                convert::from_lsp::diagnostic_to_diagnostic(span.clone(), diagnostic.clone());

            if let (
                Some(lsp_types::NumberOrString::Number(2)),
                Some(loa::Diagnostic::UndefinedTypeReference(_, s)),
            ) = (&diagnostic.code, loa_diagnostic)
            {
                let new_file_uri = uri.neighboring_file(format!("{}.loa", s).as_ref())?;
                let new_file_url = convert::from_loa::uri_to_url(&new_file_uri);
                let create_file = DocumentChangeOperation::Op(ResourceOp::Create(CreateFile {
                    uri: new_file_url.clone(),
                    options: Some(CreateFileOptions {
                        ignore_if_exists: Some(true),
                        overwrite: None,
                    }),
                }));
                let new_file_edit = DocumentChangeOperation::Edit(TextDocumentEdit {
                    text_document: VersionedTextDocumentIdentifier {
                        uri: new_file_url.clone(),
                        version: None,
                    },
                    edits: vec![TextEdit {
                        range: Default::default(),
                        new_text: match namespace {
                            None => format!("class {}.\n", s),
                            Some(ref namespace) => {
                                format!("namespace {}.\n\nclass {}.\n", namespace, s)
                            }
                        },
                    }],
                });

                let insert_import_edit = DocumentChangeOperation::Edit(TextDocumentEdit {
                    text_document: VersionedTextDocumentIdentifier {
                        uri: url.clone(),
                        version: None,
                    },
                    edits: vec![TextEdit {
                        range: convert::from_loa::span_to_range(loa::Span::new(
                            end_of_import_list.clone(),
                            end_of_import_list.clone(),
                        )),
                        new_text: match namespace {
                            None => format!("\nimport {}.", s),
                            Some(ref namespace) => format!("\nimport {}/{}.", namespace, s),
                        },
                    }],
                });

                actions.push(CodeActionOrCommand::CodeAction(CodeAction {
                    title: format!("Create class `{}`", s),
                    kind: None,
                    diagnostics: Some(vec![diagnostic]),
                    command: None,
                    edit: Some(WorkspaceEdit {
                        changes: None,
                        document_changes: Some(DocumentChanges::Operations(vec![
                            create_file,
                            new_file_edit,
                            insert_import_edit,
                        ])),
                    }),
                }))
            }
        }

        Some(actions)
    }
}
