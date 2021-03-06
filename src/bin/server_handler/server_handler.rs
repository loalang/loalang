use crate::server_handler::*;
use serde_json::Value;

pub struct ServerHandler {
    context: ServerContext,
}

impl ServerHandler {
    pub fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(
                TextDocumentSyncKind::Incremental,
            )),
            hover_provider: Some(true),
            completion_provider: Some(CompletionOptions {
                resolve_provider: Some(true),
                trigger_characters: Some(vec![" ".into()]),
            }),
            signature_help_provider: None,
            definition_provider: Some(true),
            type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
            implementation_provider: None,
            references_provider: Some(true),
            document_highlight_provider: Some(true),
            document_symbol_provider: None,
            workspace_symbol_provider: None,
            code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
            code_lens_provider: None,
            document_formatting_provider: Some(true),
            document_range_formatting_provider: None,
            document_on_type_formatting_provider: None,
            rename_provider: Some(RenameProviderCapability::Options(RenameOptions {
                prepare_provider: Some(true),
            })),
            color_provider: None,
            folding_range_provider: None,
            execute_command_provider: None,
            workspace: Some(WorkspaceCapability {
                workspace_folders: Some(WorkspaceFolderCapability {
                    change_notifications: Some(WorkspaceFolderCapabilityChangeNotifications::Bool(
                        true,
                    )),
                    supported: Some(true),
                }),
            }),
        }
    }

    pub fn new(context: ServerContext) -> ServerHandler {
        ServerHandler { context }
    }

    pub fn handle(&mut self, method: &str, params: Value) -> Result<Value, ServerError> {
        // info!("-> {:?} {:#}", method, params);

        macro_rules! handle_notification {
            ($notification:ty) => {
                if method == <<$notification as NotificationHandler>::N as notification::Notification>::METHOD {
                    if let Ok(params) = serde_json::from_value::<<<$notification as NotificationHandler>::N as notification::Notification>::Params>(params) {
                        <$notification>::handle(&mut self.context, params);
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
            ($request:ty) => {
                if method == <<$request as RequestHandler>::R as request::Request>::METHOD {
                    if let Ok(params) = serde_json::from_value::<
                        <<$request as RequestHandler>::R as request::Request>::Params,
                    >(params)
                    {
                        let r: <<$request as RequestHandler>::R as request::Request>::Result =
                            <$request>::handle(&mut self.context, params);
                        return match serde_json::to_value(r) {
                            Ok(v) => Ok(v),
                            Err(e) => Err(e.into()),
                        };
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

        handle_notification!(DidOpenTextDocumentNotificationHandler);
        handle_notification!(DidChangeTextDocumentNotificationHandler);
        handle_notification!(DidChangeWatchedFilesNotificationHandler);

        handle_request!(RenameRequestHandler);
        handle_request!(PrepareRenameRequestHandler);
        handle_request!(GotoDefinitionRequestHandler);
        handle_request!(GotoTypeDefinitionRequestHandler);
        handle_request!(ReferencesRequestHandler);
        handle_request!(CodeActionRequestHandler);
        handle_request!(CompletionRequestHandler);
        handle_request!(DocumentHighlightRequestHandler);
        handle_request!(HoverRequestHandler);
        handle_request!(FormattingRequestHandler);

        warn!("UNKNOWN MESSAGE: {}", method);

        Err(ServerError::Empty)
    }
}
