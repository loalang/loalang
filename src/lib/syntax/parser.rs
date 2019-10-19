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
    last_token_span: Span,
}

impl Parser {
    pub fn new(source: Arc<Source>) -> Parser {
        let start = Location::at_offset(&source, 0);
        Parser {
            tokens: tokenize(source)
                .into_iter()
                .filter(|token| !matches!(token.kind, Whitespace(_)))
                .collect(),
            diagnostics: vec![],
            last_token_span: Span::new(start.clone(), start),
        }
    }

    fn next(&mut self) -> Token {
        let token = self.tokens.remove(0);
        self.last_token_span = token.span.clone();
        token
    }

    fn syntax_error(&mut self, message: &str) {
        let message = String::from(message);
        let span = self.tokens[0].span.clone();

        self.diagnostics
            .push(Diagnostic::SyntaxError(span, message));
    }

    fn syntax_error_end(&mut self, message: &str) {
        let message = String::from(message);

        self.diagnostics.push(Diagnostic::SyntaxError(
            self.last_token_span.clone(),
            message,
        ));
    }

    #[inline]
    fn finalize(&self, builder: NodeBuilder, kind: NodeKind) -> Id {
        builder.finalize(self.last_token_span.end.clone(), kind)
    }

    pub fn parse(mut self) -> (Arc<Tree>, Vec<Diagnostic>) {
        let mut tree = Tree::new();
        self.parse_module(&mut tree);
        (Arc::new(tree), self.diagnostics)
    }

    #[inline]
    fn child<'a>(&self, builder: &'a mut NodeBuilder) -> NodeBuilder<'a> {
        builder.child(self.tokens[0].span.start.clone())
    }

    fn parse_module(&mut self, tree: &mut Tree) -> Id {
        let mut builder = NodeBuilder::new(tree, self.tokens[0].span.start.clone());

        let mut namespace_directive = Id::NULL;
        let mut import_directives = vec![];
        let mut module_declarations = vec![];

        if sees!(self, NamespaceKeyword) {
            namespace_directive = self.parse_namespace_directive(self.child(&mut builder));
        } else {
            self.syntax_error("Each module must start with a namespace directive.")
        }

        while sees!(self, ImportKeyword) {
            import_directives.push(self.parse_import_directive(self.child(&mut builder)));
        }

        while !sees!(self, EOF) {
            let before = self.tokens.len();
            module_declarations.push(self.parse_module_declaration(self.child(&mut builder)));
            let after = self.tokens.len();

            if before == after {
                self.syntax_error("Unexpected token.");
                self.next();
            }
        }

        self.finalize(
            builder,
            Module {
                namespace_directive,
                import_directives,
                module_declarations,
            },
        )
    }

    fn parse_namespace_directive(&mut self, mut builder: NodeBuilder) -> Id {
        let mut namespace_keyword = None;
        let mut qualified_symbol = Id::NULL;
        let mut period = None;

        if sees!(self, NamespaceKeyword) {
            namespace_keyword = Some(self.next());
        } else {
            self.syntax_error("Expected namespace keyword.");
        }

        if sees!(self, SimpleSymbol(_)) {
            qualified_symbol = self.parse_qualified_symbol(self.child(&mut builder));
        } else {
            self.syntax_error_end("Expected qualified symbol.");
        }

        if sees!(self, Period) {
            period = Some(self.next());
        } else {
            self.syntax_error_end("Namespace directive must end with period.");
        }

        self.finalize(
            builder,
            NamespaceDirective {
                namespace_keyword,
                qualified_symbol,
                period,
            },
        )
    }

    fn parse_import_directive(&mut self, mut builder: NodeBuilder) -> Id {
        let mut import_keyword = None;
        let mut qualified_symbol = Id::NULL;
        let mut as_keyword = None;
        let mut symbol = Id::NULL;
        let mut period = None;

        if sees!(self, ImportKeyword) {
            import_keyword = Some(self.next());
        } else {
            self.syntax_error("Expected import keyword.");
        }

        if sees!(self, SimpleSymbol(_)) {
            qualified_symbol = self.parse_qualified_symbol(self.child(&mut builder));
        } else {
            self.syntax_error_end("Expected qualified symbol.");
        }

        if sees!(self, AsKeyword) {
            as_keyword = Some(self.next());

            if sees!(self, SimpleSymbol(_)) {
                symbol = self.parse_symbol(self.child(&mut builder));
            } else {
                self.syntax_error_end("Expected import alias.");
            }
        }

        if sees!(self, Period) {
            period = Some(self.next());
        } else {
            self.syntax_error_end("Import directive must end with period.");
        }

        self.finalize(
            builder,
            ImportDirective {
                import_keyword,
                qualified_symbol,
                as_keyword,
                symbol,
                period,
            },
        )
    }

    fn parse_qualified_symbol(&mut self, mut builder: NodeBuilder) -> Id {
        if !sees!(self, SimpleSymbol(_)) {
            self.syntax_error("Expected qualified symbol.");
        }

        let mut symbols = vec![];
        while sees!(self, SimpleSymbol(_)) {
            symbols.push(self.parse_symbol(self.child(&mut builder)));
            if sees!(self, Slash) {
                self.next();
                if !sees!(self, SimpleSymbol(_)) {
                    self.syntax_error_end("Expected a symbol.");
                }
                continue;
            }
            break;
        }

        self.finalize(builder, QualifiedSymbol { symbols })
    }

    fn parse_symbol(&mut self, builder: NodeBuilder) -> Id {
        let token = self.next();
        self.finalize(builder, Symbol(token))
    }

    fn parse_module_declaration(&mut self, mut builder: NodeBuilder) -> Id {
        if sees!(self, ExportKeyword) {
            let export_keyword = self.next();
            let class = self.parse_class(self.child(&mut builder));
            self.finalize(builder, Exported(export_keyword, class))
        } else {
            self.parse_class(builder)
        }
    }

    fn parse_class(&mut self, mut builder: NodeBuilder) -> Id {
        let mut class_keyword = None;
        let mut symbol = Id::NULL;
        let mut class_body = Id::NULL;
        let mut period = None;

        if sees!(self, ClassKeyword) {
            class_keyword = Some(self.next());
        } else {
            self.syntax_error("Expected class keyword.");
        }

        if sees!(self, SimpleSymbol(_)) {
            symbol = self.parse_symbol(self.child(&mut builder));
        } else {
            self.syntax_error("Every class must have a name.");
        }

        if sees!(self, Period) {
            period = Some(self.next());
        } else if sees!(self, OpenCurly) {
            class_body = self.parse_class_body(self.child(&mut builder));

            if sees!(self, Period) {
                self.syntax_error("A class with a body doesn't need to end with a period.");
                self.next();
            }
        } else {
            self.syntax_error_end("Class must have a body or end with a period.");
        }

        self.finalize(
            builder,
            Class {
                class_keyword,
                symbol,
                class_body,
                period,
            },
        )
    }

    fn parse_class_body(&mut self, mut builder: NodeBuilder) -> Id {
        let mut open_curly = None;
        let mut close_curly = None;
        let mut class_members = vec![];

        if sees!(self, OpenCurly) {
            open_curly = Some(self.next());
        } else {
            self.syntax_error("Expected class body.");
        }

        while !sees!(self, CloseCurly | EOF) {
            if sees!(self, PublicKeyword | PrivateKeyword) {
                class_members.push(self.parse_method(self.child(&mut builder)));
                continue;
            }
            self.syntax_error("Expected a class member.");
            while !sees!(self, CloseCurly | EOF | PrivateKeyword | PublicKeyword) {
                self.next();
            }
        }

        if sees!(self, CloseCurly) {
            close_curly = Some(self.next());
        } else {
            self.syntax_error("Expected end of class body.");
        }

        self.finalize(
            builder,
            ClassBody {
                open_curly,
                class_members,
                close_curly,
            },
        )
    }

    fn parse_method(&mut self, mut builder: NodeBuilder) -> Id {
        let mut visibility = None;
        let mut signature = Id::NULL;
        let mut method_body = Id::NULL;
        let mut period = None;

        if sees!(self, PrivateKeyword | PublicKeyword) {
            visibility = Some(self.next());
        }

        signature = self.parse_signature(self.child(&mut builder));

        if sees!(self, FatArrow) {
            method_body = self.parse_method_body(self.child(&mut builder));
        }

        if sees!(self, Period) {
            period = Some(self.next());
        } else {
            self.syntax_error_end("Methods must end with a period.");
        }

        self.finalize(
            builder,
            Method {
                visibility,
                signature,
                method_body,
                period,
            },
        )
    }

    fn parse_signature(&mut self, mut builder: NodeBuilder) -> Id {
        let mut message_pattern = Id::NULL;
        let mut return_type = Id::NULL;

        message_pattern = self.parse_message_pattern(self.child(&mut builder));

        if sees!(self, Arrow) {
            return_type = self.parse_return_type(self.child(&mut builder));
        }

        self.finalize(
            builder,
            Signature {
                message_pattern,
                return_type,
            },
        )
    }

    fn parse_message_pattern(&mut self, builder: NodeBuilder) -> Id {
        if sees!(self, SimpleSymbol(_)) {
            if matches!(self.tokens[1].kind, Colon) {
                self.parse_keyword_message_pattern(builder)
            } else {
                self.parse_unary_message_pattern(builder)
            }
        } else if sees!(self, Plus | Slash | EqualSign | OpenAngle | CloseAngle) {
            self.parse_binary_message_pattern(builder)
        } else {
            self.syntax_error_end("Expected symbol or operator.");
            Id::NULL
        }
    }

    fn parse_unary_message_pattern(&mut self, mut builder: NodeBuilder) -> Id {
        let mut symbol = Id::NULL;

        if sees!(self, SimpleSymbol(_)) {
            symbol = self.parse_symbol(self.child(&mut builder));
        } else {
            self.syntax_error("Expected symbol.");
        }

        self.finalize(builder, UnaryMessagePattern { symbol })
    }

    fn parse_binary_message_pattern(&mut self, mut builder: NodeBuilder) -> Id {
        let mut operator = Id::NULL;
        let mut parameter_pattern = Id::NULL;

        if sees!(self, Plus | Slash | EqualSign | OpenAngle | CloseAngle) {
            operator = self.parse_operator(self.child(&mut builder));
        } else {
            self.syntax_error("Expected operator.");
        }

        if sees!(self, SimpleSymbol(_)) {
            parameter_pattern = self.parse_parameter_pattern(self.child(&mut builder));
        } else {
            self.syntax_error("Expected parameter pattern.");
        }

        self.finalize(
            builder,
            BinaryMessagePattern {
                operator,
                parameter_pattern,
            },
        )
    }

    fn parse_keyword_message_pattern(&mut self, mut builder: NodeBuilder) -> Id {
        let mut keyword_pairs = vec![];

        if !sees!(self, SimpleSymbol(_)) {
            self.syntax_error("Expected keywords.");
            return Id::NULL;
        }

        while sees!(self, SimpleSymbol(_)) {
            keyword_pairs.push(
                self.parse_keyword_pair(self.child(&mut builder), Self::parse_parameter_pattern),
            );
        }

        self.finalize(builder, KeywordMessagePattern { keyword_pairs })
    }

    #[inline]
    fn parse_keyword_pair<F: FnOnce(&mut Self, NodeBuilder) -> Id>(
        &mut self,
        mut builder: NodeBuilder,
        f: F,
    ) -> Id {
        let mut keyword = Id::NULL;
        let mut colon = None;
        let mut value = Id::NULL;

        if sees!(self, SimpleSymbol(_)) {
            keyword = self.parse_symbol(self.child(&mut builder));
        } else {
            self.syntax_error("Expected symbol.");
            return Id::NULL;
        }

        if sees!(self, Colon) {
            colon = Some(self.next());
        } else {
            self.syntax_error_end("Expected colon.");
        }

        let child_builder = self.child(&mut builder);
        value = f(self, child_builder);

        self.finalize(
            builder,
            KeywordPair {
                keyword,
                colon,
                value,
            },
        )
    }

    fn parse_method_body(&mut self, mut builder: NodeBuilder) -> Id {
        Id::NULL
    }

    fn parse_return_type(&mut self, mut builder: NodeBuilder) -> Id {
        let mut arrow = None;
        let mut type_expression = Id::NULL;

        if sees!(self, Arrow) {
            arrow = Some(self.next());
        } else {
            self.syntax_error("Expected return type.");
        }

        type_expression = self.parse_type_expression(self.child(&mut builder));

        self.finalize(
            builder,
            ReturnType {
                arrow,
                type_expression,
            },
        )
    }

    fn parse_type_expression(&mut self, mut builder: NodeBuilder) -> Id {
        if sees!(self, SimpleSymbol(_)) {
            self.parse_reference_type_expression(builder)
        } else {
            self.syntax_error("Expected type expression.");
            Id::NULL
        }
    }

    fn parse_reference_type_expression(&mut self, mut builder: NodeBuilder) -> Id {
        let mut symbol = Id::NULL;

        if sees!(self, SimpleSymbol(_)) {
            symbol = self.parse_symbol(self.child(&mut builder));
        } else {
            self.syntax_error("Expected type name.");
        }

        self.finalize(builder, ReferenceTypeExpression { symbol })
    }

    fn parse_operator(&mut self, mut builder: NodeBuilder) -> Id {
        if !sees!(self, Plus | Slash | EqualSign | OpenAngle | CloseAngle) {
            return Id::NULL;
        }

        let token = self.next();

        self.finalize(builder, Operator(token))
    }

    fn parse_parameter_pattern(&mut self, mut builder: NodeBuilder) -> Id {
        let mut type_expression = Id::NULL;
        let mut symbol = Id::NULL;

        self.finalize(
            builder,
            ParameterPattern {
                type_expression,
                symbol,
            },
        )
    }
}
