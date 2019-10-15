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

        if sees!(self, NamespaceKeyword) {
            module.namespace_directive = Some(self.parse_namespace_directive());
        } else {
            self.syntax_error("Each module must start with a namespace directive.")
        }

        while sees!(self, ImportKeyword) {
            module.import_directives.push(self.parse_import_directive());
        }

        while !sees!(self, EOF) {
            let before = self.tokens.len();
            module
                .module_declarations
                .push(self.parse_module_declaration());
            let after = self.tokens.len();

            if before == after {
                self.syntax_error("Unexpected token.");
                self.next();
            }
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

    pub fn parse_import_directive(&mut self) -> ImportDirective {
        let mut directive = ImportDirective {
            id: Id::new(),
            import_keyword: None,
            qualified_symbol: None,
            as_keyword: None,
            symbol: None,
            period: None,
        };

        if sees!(self, ImportKeyword) {
            directive.import_keyword = Some(self.next());
        } else {
            self.syntax_error("Expected import keyword.");
        }

        if sees!(self, SimpleSymbol(_)) {
            directive.qualified_symbol = Some(self.parse_qualified_symbol());
        } else {
            self.syntax_error("Expected qualified symbol to import.");
        }

        if sees!(self, AsKeyword) {
            directive.as_keyword = Some(self.next());

            if sees!(self, SimpleSymbol(_)) {
                directive.symbol = Some(self.parse_symbol());
            }
        }

        if sees!(self, Period) {
            directive.period = Some(self.next());
        } else {
            self.syntax_error("Import directive must end with a period.");
        }

        directive
    }

    pub fn parse_module_declaration(&mut self) -> ModuleDeclaration {
        if sees!(self, ExportKeyword) {
            ModuleDeclaration::Exported(self.next(), self.parse_declaration())
        } else {
            ModuleDeclaration::NotExported(self.parse_declaration())
        }
    }

    pub fn parse_declaration(&mut self) -> Declaration {
        Declaration::Class(self.parse_class())
    }

    pub fn parse_class(&mut self) -> Class {
        let mut class = Class {
            id: Id::new(),
            partial_keyword: None,
            class_keyword: None,
            symbol: None,
            body: None,
            period: None,
        };

        if sees!(self, PartialKeyword) {
            class.partial_keyword = Some(self.next());
        }

        if sees!(self, ClassKeyword) {
            class.class_keyword = Some(self.next());
        } else {
            self.syntax_error("Expected class keyword.");
        }

        if sees!(self, SimpleSymbol(_)) {
            class.symbol = Some(self.parse_symbol());
        } else {
            self.syntax_error("Classes must have names.");
        }

        if !sees!(self, OpenCurly | Period) {
            self.syntax_error("Expected a class body or a period.");
        }

        if sees!(self, OpenCurly) {
            class.body = Some(self.parse_class_body());
        }

        if sees!(self, Period) {
            if class.body.is_some() {
                self.syntax_error("A class with a body doesn't need to end with a period.");
            }
            class.period = Some(self.next());
        }

        class
    }

    pub fn parse_class_body(&mut self) -> ClassBody {
        let mut class_body = ClassBody {
            id: Id::new(),
            open_curly: None,
            class_members: vec![],
            close_curly: None,
        };

        if sees!(self, OpenCurly) {
            class_body.open_curly = Some(self.next());
        } else {
            self.syntax_error("Expected a class body.");
        }

        while !sees!(self, EOF | CloseCurly) {
            let before = self.tokens.len();
            class_body.class_members.push(self.parse_class_member());
            let after = self.tokens.len();

            if before == after {
                break;
            }
        }

        if sees!(self, CloseCurly) {
            class_body.close_curly = Some(self.next());
        } else {
            self.syntax_error("Unterminated class body.")
        }

        class_body
    }

    pub fn parse_class_member(&mut self) -> ClassMember {
        ClassMember::Method(self.parse_method())
    }

    pub fn parse_method(&mut self) -> Method {
        let mut visibility = None;

        if sees!(self, PublicKeyword | PrivateKeyword) {
            visibility = Some(self.next());
        } else {
            self.syntax_error("Methods must be designated as public or private.");
        }

        let mut method = Method {
            id: Id::new(),
            visibility,
            signature: self.parse_signature(),
            body: None,
            period: None,
        };

        if sees!(self, FatArrow) {
            method.body = Some(self.parse_method_body());
        }

        if sees!(self, Period) {
            method.period = Some(self.next());
        } else {
            self.syntax_error("Methods must be terminated with a period.");
        }

        method
    }

    pub fn parse_signature(&mut self) -> Signature {
        let mut signature = Signature {
            id: Id::new(),
            message_pattern: None,
            return_type: None,
        };

        if sees!(self, SimpleSymbol(_)) || self.sees_operator() {
            signature.message_pattern = self.parse_message_pattern();
        } else {
            self.syntax_error("Expected message pattern.");
        }

        if sees!(self, Arrow) {
            signature.return_type = Some(self.parse_return_type());
        }

        signature
    }

    pub fn parse_return_type(&mut self) -> ReturnType {
        let mut return_type = ReturnType {
            id: Id::new(),
            arrow: None,
            type_expression: None,
        };

        if sees!(self, Arrow) {
            return_type.arrow = Some(self.next());

            return_type.type_expression = self.parse_type_expression();
        } else {
            self.syntax_error("Expected return type.");
        }

        return_type
    }

    pub fn parse_type_expression(&mut self) -> Option<TypeExpression> {
        if sees!(self, SimpleSymbol(_)) {
            Some(TypeExpression::Reference(Id::new(), self.parse_symbol()))
        } else {
            None
        }
    }

    fn sees_operator(&self) -> bool {
        sees!(self, Plus | Slash | EqualSign | OpenAngle | CloseAngle)
    }

    pub fn parse_message_pattern(&mut self) -> Option<MessagePattern> {
        if sees!(self, SimpleSymbol(_)) {
            let symbol = self.parse_symbol();
            if sees!(self, Colon) {
                let mut keyworded = Keyworded {
                    id: Id::new(),
                    keywords: vec![(symbol, self.next(), self.parse_parameter_pattern())],
                };

                while sees!(self, SimpleSymbol(_)) {
                    keyworded.keywords.push((
                        self.parse_symbol(),
                        {
                            if !sees!(self, Colon) {
                                self.syntax_error("Expected colon.");
                            }
                            self.next()
                        },
                        self.parse_parameter_pattern(),
                    ));
                }

                Some(MessagePattern::Keyword(Id::new(), keyworded))
            } else {
                Some(MessagePattern::Unary(Id::new(), symbol))
            }
        } else if self.sees_operator() {
            Some(MessagePattern::Binary(
                Id::new(),
                self.next(),
                self.parse_parameter_pattern(),
            ))
        } else {
            None
        }
    }

    pub fn parse_method_body(&mut self) -> MethodBody {
        let mut method_body = MethodBody {
            id: Id::new(),
            fat_arrow: None,
            expression: None,
        };

        if sees!(self, FatArrow) {
            method_body.fat_arrow = Some(self.next());
        } else {
            self.syntax_error("Expected a method body.");
        }

        if self.sees_expression() {
            method_body.expression = self.parse_expression();
        } else {
            self.syntax_error("Expected an expression.");
        }

        method_body
    }

    fn sees_expression(&self) -> bool {
        sees!(self, SimpleSymbol(_))
    }

    pub fn parse_expression(&mut self) -> Option<Expression> {
        if sees!(self, SimpleSymbol(_)) {
            Some(Expression::Reference(Id::new(), self.parse_symbol()))
        } else {
            None
        }
    }

    pub fn parse_parameter_pattern(&mut self) -> ParameterPattern {
        if sees!(self, Underscore) {
            ParameterPattern::Nothing(Id::new(), self.next())
        } else if sees!(self, SimpleSymbol(_)) {
            ParameterPattern::Parameter(
                Id::new(),
                self.parse_type_expression(),
                Some(self.parse_symbol()),
            )
        } else {
            self.syntax_error("Expected a parameter");
            ParameterPattern::Nothing(Id::new(), self.next())
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
