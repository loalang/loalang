use loa::Location;
use loa::*;
use lsp_types::*;
use serde_json::Value;

pub struct ServerHandler {
    program_cell: ProgramCell,
    pub capabilities: ServerCapabilities,
}

impl ServerHandler {
    pub fn new() -> ServerHandler {
        ServerHandler {
            program_cell: ProgramCell::new(),
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
                references_provider: None,
                document_highlight_provider: None,
                document_symbol_provider: None,
                workspace_symbol_provider: None,
                code_action_provider: None,
                code_lens_provider: None,
                document_formatting_provider: None,
                document_range_formatting_provider: None,
                document_on_type_formatting_provider: None,
                rename_provider: None,
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
                    return Err(ServerError::none());
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
                        return Err(ServerError::none());
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
            get_definition
        );

        warn!("UNKNOWN MESSAGE: {}", method);

        Err(ServerError::none())
    }

    fn uri_from_url(url: &Url) -> URI {
        URI::Exact(url.as_str().into())
    }

    fn maybe_span_from_range(source: &Arc<Source>, range: Option<Range>) -> Option<Span> {
        range.map(|r| Self::span_from_range(source, r))
    }

    fn span_from_range(source: &Arc<Source>, range: Range) -> Span {
        Span::new(
            Self::location_from_position(source, range.start),
            Self::location_from_position(source, range.end),
        )
    }

    fn location_from_position(source: &Arc<Source>, pos: Position) -> Location {
        Location::at_position(source, pos.line as usize + 1, pos.character as usize + 1)
    }

    fn location_from_span(span: Span) -> lsp_types::Location {
        lsp_types::Location {
            uri: Self::url_from_uri(&span.start.uri),
            range: Self::range_from_span(span),
        }
    }

    fn url_from_uri(uri: &URI) -> Url {
        Url::parse(format!("{}", uri).as_str()).unwrap()
    }

    fn range_from_span(span: Span) -> Range {
        Range {
            start: Self::position_from_location(span.start),
            end: Self::position_from_location(span.end),
        }
    }

    fn position_from_location(location: Location) -> Position {
        Position {
            line: (location.line - 1) as u64,
            character: (location.character - 1) as u64,
        }
    }

    fn location_from_position_params(
        &self,
        params: TextDocumentPositionParams,
    ) -> Option<Location> {
        let uri = Self::uri_from_url(&params.text_document.uri);
        let module_cell = self.program_cell.get(&uri)?;
        Some(Self::location_from_position(
            &module_cell.source,
            params.position,
        ))
    }

    pub fn did_open_text_document(&mut self, params: DidOpenTextDocumentParams) {
        let uri = Self::uri_from_url(&params.text_document.uri);
        let source = Source::new(uri, params.text_document.text);
        self.program_cell.set(source);
    }

    pub fn did_change_text_document(&mut self, params: DidChangeTextDocumentParams) {
        let uri = Self::uri_from_url(&params.text_document.uri);
        if let Some(module_cell) = self.program_cell.get_mut(&uri) {
            for change in params.content_changes {
                match Self::maybe_span_from_range(&module_cell.source, change.range) {
                    None => module_cell.replace(change.text),
                    Some(span) => module_cell.change(span, change.text.as_ref()),
                }
            }
        }
    }

    pub fn get_definition(
        &mut self,
        params: TextDocumentPositionParams,
    ) -> Result<lsp_types::Location, ServerError> {
        let location = self.location_from_position_params(params)?;
        let selection = self.program_cell.declaration(location);
        Ok(Self::location_from_span(selection.span()?))
    }
}

#[derive(Debug)]
pub struct ServerError(pub i32, pub String);

impl ServerError {
    pub fn none() -> ServerError {
        std::option::NoneError.into()
    }
}

impl Error for ServerError {}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.1)
    }
}

impl From<std::option::NoneError> for ServerError {
    fn from(_: std::option::NoneError) -> Self {
        ServerError(0, "Empty result".into())
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(_: serde_json::Error) -> Self {
        ServerError(1, "Failed (un)serialization".into())
    }
}
