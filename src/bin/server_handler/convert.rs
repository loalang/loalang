pub mod from_loa {
    use loa;
    use lsp_types as lsp;

    pub fn span_to_location(span: loa::Span) -> lsp::Location {
        lsp::Location {
            uri: uri_to_url(&span.start.uri),
            range: span_to_range(span),
        }
    }

    pub fn uri_to_url(uri: &loa::URI) -> lsp::Url {
        lsp::Url::parse(format!("{}", uri).as_str()).unwrap()
    }

    pub fn span_to_range(span: loa::Span) -> lsp::Range {
        lsp::Range {
            start: location_to_position(span.start),
            end: location_to_position(span.end),
        }
    }

    pub fn location_to_position(location: loa::Location) -> lsp::Position {
        lsp::Position {
            line: (location.line - 1) as u64,
            character: (location.character - 1) as u64,
        }
    }

    pub fn diagnostics_to_diagnostics(diagnostics: Vec<loa::Diagnostic>) -> Vec<lsp::Diagnostic> {
        diagnostics
            .into_iter()
            .map(diagnostic_to_diagnostic)
            .collect()
    }

    pub fn diagnostic_to_diagnostic(diagnostic: loa::Diagnostic) -> lsp::Diagnostic {
        lsp::Diagnostic {
            range: span_to_range(diagnostic.span().clone()),
            severity: Some(level_to_severity(diagnostic.level())),
            code: Some(lsp::NumberOrString::Number(diagnostic.code() as u64)),
            source: None,
            message: diagnostic.to_string(),
            related_information: None,
        }
    }

    pub fn level_to_severity(level: loa::DiagnosticLevel) -> lsp::DiagnosticSeverity {
        match level {
            loa::DiagnosticLevel::Error => lsp::DiagnosticSeverity::Error,
            loa::DiagnosticLevel::Warning => lsp::DiagnosticSeverity::Warning,
            loa::DiagnosticLevel::Info => lsp::DiagnosticSeverity::Information,
        }
    }
}

pub mod from_lsp {
    use loa;
    use lsp_types as lsp;

    pub fn url_to_uri(url: &lsp::Url) -> loa::URI {
        loa::URI::Exact(url.as_str().into())
    }

    pub fn position_params(params: lsp::TextDocumentPositionParams) -> (loa::URI, (usize, usize)) {
        let uri = url_to_uri(&params.text_document.uri);
        (uri, position(params.position))
    }

    pub fn range(range: lsp::Range) -> ((usize, usize), (usize, usize)) {
        (position(range.start), position(range.end))
    }

    pub fn position(position: lsp::Position) -> (usize, usize) {
        (position.line as usize + 1, position.character as usize + 1)
    }

    pub fn diagnostic_to_diagnostic(
        span: loa::Span,
        diagnostic: lsp::Diagnostic,
    ) -> Option<loa::Diagnostic> {
        match diagnostic.code {
            Some(lsp::NumberOrString::Number(2)) => Some(loa::Diagnostic::UndefinedTypeReference(
                span,
                diagnostic.message[1..diagnostic.message.len() - 15].into(),
            )),
            Some(lsp::NumberOrString::Number(3)) => Some(loa::Diagnostic::UndefinedReference(
                span,
                diagnostic.message[1..diagnostic.message.len() - 15].into(),
            )),
            _ => None,
        }
    }
}
