use loa::*;
use lsp_types::*;
use serde_json::Value;

mod server_error;

pub use self::server_error::*;

mod convert;

pub struct ServerHandler<F> {
    program: program::Program,
    pub capabilities: ServerCapabilities,
    notify: F,
}

impl<F> ServerHandler<F>
where
    F: Fn(String, Value) -> (),
{
    pub fn new(notify: F) -> ServerHandler<F> {
        ServerHandler {
            notify,
            program: program::new(),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Incremental,
                )),
                hover_provider: None,
                completion_provider: None,
                signature_help_provider: None,
                definition_provider: Some(true),
                type_definition_provider: None,
                implementation_provider: None,
                references_provider: Some(true),
                document_highlight_provider: None,
                document_symbol_provider: None,
                workspace_symbol_provider: None,
                code_action_provider: None,
                code_lens_provider: None,
                document_formatting_provider: None,
                document_range_formatting_provider: None,
                document_on_type_formatting_provider: None,
                rename_provider: Some(RenameProviderCapability::Options(RenameOptions {
                    prepare_provider: Some(true),
                })),
                color_provider: None,
                folding_range_provider: None,
                execute_command_provider: None,
                workspace: None,
            },
        }
    }

    pub fn handle(&mut self, method: &str, params: Value) -> Result<Value, ServerError> {
        macro_rules! handle_notification {
            ($match:expr, $t:ty, $f:ident) => {
                if method == $match {
                    if let Ok(params) = serde_json::from_value::<$t>(params) {
                        self.$f(params);
                    } else {
                        error!(
                            "Failed to deserialize method params for notification: {}",
                            method
                        );
                    }
                    return Err(ServerError::Empty);
                }
            };
        }
        macro_rules! handle_request {
            ($match:expr, $t:ty, $f:ident) => {
                if method == $match {
                    if let Ok(params) = serde_json::from_value::<$t>(params) {
                        return self.$f(params).and_then(|v| match serde_json::to_value(v) {
                            Ok(v) => Ok(v),
                            Err(e) => Err(e.into()),
                        });
                    } else {
                        error!(
                            "Failed to deserialize method params for request: {}",
                            method
                        );
                        return Err(ServerError::Empty);
                    }
                }
            };
        }

        handle_notification!(
            "textDocument/didChange",
            DidChangeTextDocumentParams,
            did_change_text_document
        );

        handle_notification!(
            "textDocument/didOpen",
            DidOpenTextDocumentParams,
            did_open_text_document
        );

        handle_request!(
            "textDocument/definition",
            TextDocumentPositionParams,
            definition
        );

        handle_request!(
            "textDocument/prepareRename",
            TextDocumentPositionParams,
            prepare_rename
        );

        handle_request!("textDocument/rename", RenameParams, rename);

        warn!("UNKNOWN MESSAGE: {}", method);

        Err(ServerError::Empty)
    }

    fn did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        let uri = convert::from_lsp::url_to_uri(&params.text_document.uri);
        self.program.set(uri, params.text_document.text);
        self.publish_updated_diagnostics();
    }

    fn did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        let uri = convert::from_lsp::url_to_uri(&params.text_document.uri);
        self.program.edit(
            params
                .content_changes
                .into_iter()
                .filter_map(|change| {
                    let range = change.range?;
                    let range = (
                        (range.start.line as usize, range.start.character as usize),
                        (range.end.line as usize, range.end.character as usize),
                    );
                    Some(program::Edit(uri.clone(), range, change.text))
                })
                .collect(),
        );
        self.publish_updated_diagnostics();
    }

    fn publish_updated_diagnostics(&mut self) {
        let diagnostics = self.program.diagnostics();

        for (uri, diagnostics) in diagnostics {
            let params = PublishDiagnosticsParams {
                uri: convert::from_loa::uri_to_url(&uri),
                diagnostics: convert::from_loa::diagnostics_to_diagnostics(diagnostics),
            };

            (self.notify)(
                "textDocument/publishDiagnostics".into(),
                serde_json::to_value(params).unwrap(),
            );
        }
    }

    fn definition(
        &mut self,
        params: TextDocumentPositionParams,
    ) -> Result<lsp_types::Location, ServerError> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let usage = self.program.usage(uri, location)?;
        Ok(convert::from_loa::span_to_location(
            usage.declaration.name_span,
        ))
    }

    fn prepare_rename(
        &mut self,
        params: TextDocumentPositionParams,
    ) -> Result<PrepareRenameResponse, ServerError> {
        let (uri, location) = convert::from_lsp::position_params(params);
        let usage = self.program.usage(uri, location)?;
        Ok(PrepareRenameResponse::RangeWithPlaceholder {
            range: convert::from_loa::span_to_range(usage.declaration.name_span),
            placeholder: usage.declaration.name,
        })
    }

    fn rename(&mut self, params: RenameParams) -> Result<WorkspaceEdit, ServerError> {
        let (uri, location) = convert::from_lsp::position_params(params.text_document_position);
        let usage = self.program.usage(uri.clone(), location)?;

        let mut edits = HashMap::new();

        for node in usage.named_nodes() {
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
            .map(|(uri, edits)| DocumentChangeOperation::Edit(TextDocumentEdit {
                text_document: VersionedTextDocumentIdentifier {
                    version: None,
                    uri: convert::from_loa::uri_to_url(&uri),
                },
                edits,
            }))
            .collect();

        Ok(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Operations(operations)),
        })
    }
}
