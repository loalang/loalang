use self::TokenKind::*;
use crate::syntax::*;
use crate::*;

macro_rules! sees {
    ($self:expr, $($pattern:tt)+) => {
        match &$self.tokens[0].kind {
            $($pattern)+ => true,
            _ => false,
        }
    };
}

pub struct Parser {
    tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Parser {
    pub fn new(source: Arc<Source>) -> Parser {
        Parser {
            tokens: tokenize(source)
                .into_iter()
                .filter(|token| !matches!(token.kind, Whitespace(_)))
                .collect(),
            diagnostics: vec![],
        }
    }

    fn next(&mut self) -> Token {
        self.tokens.remove(0)
    }

    pub fn is_at_end(&self) -> bool {
        matches!(self.tokens[0].kind, TokenKind::EOF)
    }

    fn syntax_error(&mut self, message: &str) {
        let message = String::from(message);
        let span = self.tokens[0].span.clone();

        self.diagnostics
            .push(Diagnostic::SyntaxError(span, message));
    }

    pub fn parse_module(&mut self) -> Module {
        let mut module = Module {
            id: Id::new(),
            namespace_directive: None,
            import_directives: vec![],
            module_declarations: vec![],
        };

        if sees!(self, NamespaceDirective) {
            module.namespace_directive = Some(self.parse_namespace_directive());
        } else {
            self.syntax_error("Each module must start with a namespace directive.")
        }

        while !sees!(self, EOF) {
            self.next();
        }

        module
    }

    pub fn parse_namespace_directive(&mut self) -> NamespaceDirective {
        let mut directive = NamespaceDirective {
            id: Id::new(),
            namespace_keyword: None,
            qualified_symbol: None,
            period: None,
        };

        if sees!(self, NamespaceKeyword) {
            directive.namespace_keyword = Some(self.next());
        } else {
            self.syntax_error("Expected keyword `namespace`.");
        }

        if sees!(self, SimpleSymbol(_)) {
            directive.qualified_symbol = Some(self.parse_qualified_symbol());
        } else {
            self.syntax_error("Expected a qualified symbol.");
        }

        if sees!(self, Period) {
            directive.period = Some(self.next());
        } else {
            self.syntax_error("Namespace directive must end with a period.");
        }

        directive
    }

    pub fn parse_qualified_symbol(&mut self) -> QualifiedSymbol {
        let mut symbol = QualifiedSymbol {
            id: Id::new(),
            symbols: vec![],
        };

        while sees!(self, SimpleSymbol(_)) {
            symbol.symbols.push(self.parse_symbol());

            if sees!(self, Slash) {
                self.next();
            } else if sees!(self, Comma) {
                self.next();
                self.syntax_error("Each symbol must be separated by a slash.")
            } else if sees!(self, SimpleSymbol(_)) {
                self.syntax_error("Each symbol must be separated by a slash.")
            }
        }

        if symbol.symbols.len() == 0 {
            self.syntax_error("Expected a symbol.")
        }

        symbol
    }

    pub fn parse_symbol(&mut self) -> Symbol {
        Symbol {
            id: Id::new(),
            token: self.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse<R, F: FnOnce(&mut Parser) -> R>(f: F, code: &str) -> R {
        let mut parser = Parser::new(Source::test(code));
        f(&mut parser)
    }

    fn assert_parses_to_end<R, F: FnOnce(&mut Parser) -> R>(f: F, code: &str) {
        let mut parser = Parser::new(Source::test(code));
        f(&mut parser);
        assert!(parser.is_at_end());
    }

    #[test]
    fn parses_any_module() {
        assert_parses_to_end(Parser::parse_module, "");
        assert_parses_to_end(Parser::parse_module, "lkjhasdf");
        assert_parses_to_end(Parser::parse_module, "namespace X.");
    }

    #[test]
    fn parses_any_namespace_directive() {
        assert_parses_to_end(Parser::parse_namespace_directive, "");
        assert_parses_to_end(Parser::parse_namespace_directive, "A/B/C");
        assert_parses_to_end(Parser::parse_namespace_directive, "namespace A/B/C");
        assert_parses_to_end(Parser::parse_namespace_directive, "namespace A/B/C.");
        assert_parses_to_end(Parser::parse_namespace_directive, "namespace A/B C.");
        assert_parses_to_end(Parser::parse_namespace_directive, "namespace A/B,C.");
    }
}
