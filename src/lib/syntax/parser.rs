use self::TokenKind::*;
use crate::syntax::*;
use crate::*;
use num_traits::pow::Pow;

macro_rules! sees {
    ($self:expr, $($pattern:tt)+) => {
        match &$self.tokens[0].kind {
            $($pattern)+ => true,
            _ => false,
        }
    };
}

#[derive(Clone)]
pub struct Parser {
    source: Arc<Source>,
    tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
    last_token_span: Span,
    leading_insignificants: Vec<Token>,

    #[cfg(test)]
    test_comments: Vec<Token>,
}

impl Parser {
    pub fn new(source: Arc<Source>) -> Parser {
        let start = Location::at_offset(&source, 0);
        let mut parser = Parser {
            source: source.clone(),
            tokens: tokenize(source),
            diagnostics: vec![],
            last_token_span: Span::new(start.clone(), start),
            leading_insignificants: vec![],

            #[cfg(test)]
            test_comments: vec![],
        };
        parser.move_past_insignificants();
        parser
    }

    fn next(&mut self) -> Token {
        let before = std::mem::replace(&mut self.leading_insignificants, vec![]);
        let mut token = self.next_insignificant();
        self.last_token_span = token.span.clone();
        token.before = before;
        self.move_past_insignificants();
        token.after = self.leading_insignificants.clone();
        token
    }

    #[inline]
    fn peek(&self) -> &Token {
        &self.tokens[0]
    }

    #[inline]
    fn peek_next_significant(&self) -> &Token {
        for significant in self.tokens.iter().skip(1) {
            match significant.kind {
                Whitespace(_) | LineComment(_) => continue,
                _ => return significant,
            }
        }
        panic!("Somehow EOF was not in tokens list.");
    }

    fn next_insignificant(&mut self) -> Token {
        self.tokens.remove(0)
    }

    fn move_past_insignificants(&mut self) {
        while sees!(self, Whitespace(_) | LineComment(_)) {
            let insignificant = self.next_insignificant();
            #[cfg(test)]
            {
                if let TokenKind::LineComment(ref content) = insignificant.kind {
                    if content.starts_with("$") {
                        self.test_comments.push(insignificant.clone());
                    }
                }
            }
            self.leading_insignificants.push(insignificant);
        }
    }

    fn syntax_error(&mut self, message: &str) {
        let message = String::from(message);
        let span = self.peek().span.clone();

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
        let tree = self.parse_root();
        (Arc::new(tree), self.diagnostics)
    }

    #[cfg(test)]
    pub fn parse_with_test_comments(mut self) -> (Arc<Tree>, Vec<Diagnostic>, Vec<Token>) {
        let tree = self.parse_root();
        (Arc::new(tree), self.diagnostics, self.test_comments)
    }

    fn parse_root(&mut self) -> Tree {
        let mut tree = Tree::new(self.source.clone());
        let builder = NodeBuilder::new(&mut tree, self.peek().span.start.clone());
        match self.source.kind {
            SourceKind::Module => {
                self.parse_module(builder);
            }
            SourceKind::REPLLine => {
                self.parse_repl_line(builder);
            }
        }
        tree
    }

    #[inline]
    fn child<'a>(&self, builder: &'a mut NodeBuilder) -> NodeBuilder<'a> {
        builder.child(self.peek().span.start.clone())
    }

    fn parse_repl_line(&mut self, mut builder: NodeBuilder) -> Id {
        let mut statements = vec![];

        while !sees!(self, EOF) {
            let before = self.tokens.len();
            statements.push(self.parse_repl_statement(self.child(&mut builder)));
            let after = self.tokens.len();

            if before == after {
                self.syntax_error("Unexpected token.");
                self.next();
            }
        }

        self.finalize(builder, REPLLine { statements })
    }

    fn parse_repl_statement(&mut self, builder: NodeBuilder) -> Id {
        if sees!(self, LetKeyword) {
            self.parse_let_binding(builder)
        } else if sees!(self, Colon) {
            self.parse_repl_directive(builder)
        } else if sees!(self, ImportKeyword) {
            self.parse_import_directive(builder)
        } else if sees!(self, PartialKeyword | ClassKeyword) {
            self.parse_class(Id::NULL, builder)
        } else {
            self.parse_repl_expression(builder)
        }
    }

    fn parse_repl_directive(&mut self, mut builder: NodeBuilder) -> Id {
        let colon = self.next();
        if !sees!(self, SimpleSymbol(_)) {
            self.syntax_error_end("Expected symbol.");
            return Id::NULL;
        }
        let symbol = self.parse_symbol(self.child(&mut builder));

        let mut expression = Id::NULL;
        let mut period = None;

        if !sees!(self, EOF | Period) {
            expression = self.parse_expression(self.child(&mut builder));
        }

        if sees!(self, Period) {
            period = Some(self.next());
        }

        self.finalize(
            builder,
            REPLDirective {
                colon,
                symbol,
                expression,
                period,
            },
        )
    }

    fn parse_repl_expression(&mut self, mut builder: NodeBuilder) -> Id {
        let expression = self.parse_expression(self.child(&mut builder));
        let mut period = None;
        if sees!(self, Period) {
            period = Some(self.next());
        }
        self.finalize(builder, REPLExpression { expression, period })
    }

    fn parse_module(&mut self, mut builder: NodeBuilder) -> Id {
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
        let mut doc = Id::NULL;
        if sees!(self, DocLineMarker) {
            doc = self.parse_doc(self.child(&mut builder));
        }

        if sees!(self, ExportKeyword) {
            let export_keyword = self.next();
            let class = self.parse_class(Id::NULL, self.child(&mut builder));
            self.finalize(builder, Exported(doc, export_keyword, class))
        } else {
            self.parse_class(doc, builder)
        }
    }

    fn parse_doc(&mut self, mut builder: NodeBuilder) -> Id {
        let doc_line_marker = self.next();
        let mut blocks = vec![];

        while sees!(self, DocText(_) | DocNewLine(_) | Underscore | Asterisk | OpenBracket | CloseBracket | OpenParen | CloseParen)
        {
            blocks.push(self.parse_doc_block(self.child(&mut builder)));
        }

        self.finalize(
            builder,
            Doc {
                doc_line_marker,
                blocks,
            },
        )
    }

    fn parse_doc_block(&mut self, builder: NodeBuilder) -> Id {
        while sees!(self, DocNewLine(_)) {
            self.next();
        }
        self.parse_doc_paragraph_block(builder)
    }

    fn parse_doc_paragraph_block(&mut self, mut builder: NodeBuilder) -> Id {
        let mut elements = vec![];
        while sees!(self, DocText(_) | Underscore | DocNewLine(_) | Asterisk | OpenBracket | CloseBracket | OpenParen | CloseParen)
        {
            if self.sees_doc_block_break() {
                break;
            }
            while sees!(self, DocNewLine(_)) {
                self.next();
            }
            elements.push(self.parse_doc_element(self.child(&mut builder)));
        }
        self.finalize(builder, DocParagraphBlock { elements })
    }

    fn parse_doc_element(&mut self, mut builder: NodeBuilder) -> Id {
        if sees!(self, Underscore) {
            self.parse_doc_italic_element(builder)
        } else if sees!(self, Asterisk) {
            self.parse_doc_bold_element(builder)
        } else if sees!(self, OpenBracket) {
            let save = self.clone();
            if let Some(k) = self.parse_doc_link_element(&mut builder) {
                self.finalize(builder, k)
            } else {
                *self = save;
                self.parse_doc_text_element(builder)
            }
        } else {
            self.parse_doc_text_element(builder)
        }
    }

    fn parse_doc_link_element(&mut self, builder: &mut NodeBuilder) -> Option<NodeKind> {
        let text = self.parse_doc_link_text(self.child(builder))?;
        let mut re = Id::NULL;

        if sees!(self, OpenParen) {
            if let Some(r) = self.parse_doc_link_ref() {
                re = self.finalize(self.child(builder), r);
            }
        }

        Some(DocLinkElement(text, re))
    }

    fn parse_doc_link_text(&mut self, builder: NodeBuilder) -> Option<Id> {
        if !sees!(self, OpenBracket) {
            return None;
        }
        let open = self.next();
        let mut tokens = vec![];
        while sees!(self, DocText(_) | DocNewLine(_) | Asterisk | Underscore | OpenBracket | OpenParen | CloseParen)
        {
            if self.sees_doc_block_break() {
                return None;
            }
            if sees!(self, DocNewLine(_)) {
                self.next();
                continue;
            }
            tokens.push(self.next());
        }
        if !sees!(self, CloseBracket) {
            return None;
        }
        let close = self.next();

        Some(self.finalize(builder, DocLinkText(open, tokens, close)))
    }

    fn parse_doc_link_ref(&mut self) -> Option<NodeKind> {
        if !sees!(self, OpenParen) {
            return None;
        }
        let open = self.next();
        let mut tokens = vec![];
        while sees!(self, DocText(_) | DocNewLine(_) | Asterisk | Underscore | OpenBracket | OpenParen | CloseBracket)
        {
            if self.sees_doc_block_break() {
                return None;
            }
            if sees!(self, DocNewLine(_)) {
                self.next();
                continue;
            }
            tokens.push(self.next());
        }
        if !sees!(self, CloseParen) {
            return None;
        }
        let close = self.next();

        Some(DocLinkRef(open, tokens, close))
    }

    fn parse_doc_text_element(&mut self, builder: NodeBuilder) -> Id {
        let mut tokens = vec![self.next()];
        while sees!(self, DocText(_) | DocNewLine(_) | CloseBracket | OpenParen | CloseParen) {
            if self.sees_doc_block_break() {
                break;
            }
            if sees!(self, DocNewLine(_)) {
                self.next();
                continue;
            }
            tokens.push(self.next());
        }
        self.finalize(builder, DocTextElement(tokens))
    }

    fn sees_doc_block_break(&self) -> bool {
        sees!(self, DocNewLine(_)) && matches!(&self.peek_next_significant().kind, DocNewLine(_))
    }

    fn parse_doc_italic_element(&mut self, builder: NodeBuilder) -> Id {
        let open = self.next();
        let mut tokens = vec![];
        while sees!(self, DocText(_) | DocNewLine(_) | CloseBracket | OpenParen | CloseParen) {
            if self.sees_doc_block_break() {
                break;
            }
            if sees!(self, DocNewLine(_)) {
                self.next();
                continue;
            }
            tokens.push(self.next());
        }
        if sees!(self, Underscore) {
            let close = self.next();

            self.finalize(builder, DocItalicElement(open, tokens, close))
        } else {
            tokens.insert(0, open);
            self.finalize(builder, DocTextElement(tokens))
        }
    }

    fn parse_doc_bold_element(&mut self, builder: NodeBuilder) -> Id {
        let open = self.next();
        let mut tokens = vec![];
        while sees!(self, DocText(_) | DocNewLine(_) | CloseBracket | OpenParen | CloseParen) {
            if self.sees_doc_block_break() {
                break;
            }
            if sees!(self, DocNewLine(_)) {
                self.next();
                continue;
            }
            tokens.push(self.next());
        }
        if sees!(self, Asterisk) {
            let close = self.next();

            self.finalize(builder, DocBoldElement(open, tokens, close))
        } else {
            tokens.insert(0, open);
            self.finalize(builder, DocTextElement(tokens))
        }
    }

    fn parse_class(&mut self, mut doc: Id, mut builder: NodeBuilder) -> Id {
        let mut partial_keyword = None;
        let mut class_keyword = None;
        let mut symbol = Id::NULL;
        let mut class_body = Id::NULL;
        let mut type_parameter_list = Id::NULL;
        let mut period = None;

        if doc == Id::NULL && sees!(self, DocLineMarker) {
            doc = self.parse_doc(self.child(&mut builder));
        }

        if sees!(self, PartialKeyword) {
            partial_keyword = Some(self.next());
        }

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

        if sees!(self, OpenAngle) {
            type_parameter_list = self.parse_type_parameter_list(self.child(&mut builder));
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
                doc,
                partial_keyword,
                class_keyword,
                symbol,
                type_parameter_list,
                class_body,
                period,
            },
        )
    }

    fn parse_type_parameter_list(&mut self, mut builder: NodeBuilder) -> Id {
        let mut open_angle = None;
        let mut type_parameters = vec![];
        let mut close_angle = None;

        if sees!(self, OpenAngle) {
            open_angle = Some(self.next());
        } else {
            self.syntax_error("Expected type parameter list.");
        }

        while !sees!(self, EOF) {
            let before = self.tokens.len();
            type_parameters.push(self.parse_type_parameter(self.child(&mut builder)));
            let after = self.tokens.len();

            if before == after {
                self.syntax_error("Unexpected token.");
                self.next();
                break;
            }

            if sees!(self, Comma) {
                self.next();
            } else {
                break;
            }

            if sees!(self, CloseAngle) {
                break;
            }
        }

        if sees!(self, CloseAngle) {
            close_angle = Some(self.next());
        } else {
            self.syntax_error_end("Unterminated type parameter list.");
        }

        self.finalize(
            builder,
            TypeParameterList {
                open_angle,
                type_parameters,
                close_angle,
            },
        )
    }

    fn parse_type_parameter(&mut self, mut builder: NodeBuilder) -> Id {
        let mut symbol = Id::NULL;
        let mut variance_keyword = None;

        if sees!(self, SimpleSymbol(_)) {
            symbol = self.parse_symbol(self.child(&mut builder));
        } else {
            self.syntax_error("Expected type parameter.");
        }

        if sees!(self, InKeyword | OutKeyword | InoutKeyword) {
            variance_keyword = Some(self.next());
        }

        self.finalize(
            builder,
            TypeParameter {
                symbol,
                variance_keyword,
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
            let mut doc = Id::NULL;

            if sees!(self, DocLineMarker) {
                doc = self.parse_doc(self.child(&mut builder));
            }

            if sees!(self, PublicKeyword | PrivateKeyword) {
                match self.peek_next_significant().kind {
                    InitKeyword => {
                        class_members.push(self.parse_initializer(doc, self.child(&mut builder)));
                    }
                    VarKeyword => {
                        class_members.push(self.parse_variable(doc, self.child(&mut builder)));
                    }
                    _ => {
                        class_members.push(self.parse_method(doc, self.child(&mut builder)));
                    }
                }
                continue;
            }

            if sees!(self, IsKeyword) {
                class_members.push(self.parse_is_directive(self.child(&mut builder)));
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

    fn parse_is_directive(&mut self, mut builder: NodeBuilder) -> Id {
        if !sees!(self, IsKeyword) {
            return Id::NULL;
        }

        let is_keyword = self.next();
        let type_expression = self.parse_type_expression(self.child(&mut builder));
        let mut period = None;

        if sees!(self, Period) {
            period = Some(self.next());
        } else {
            self.syntax_error_end("Directive must end with a period.");
        }

        self.finalize(
            builder,
            IsDirective {
                is_keyword,
                type_expression,
                period,
            },
        )
    }

    fn parse_variable(&mut self, mut doc: Id, mut builder: NodeBuilder) -> Id {
        let mut visibility = None;
        let mut var_keyword = None;
        let mut type_expression = Id::NULL;
        let symbol;
        let mut equal_sign = None;
        let mut expression = Id::NULL;
        let mut period = None;

        if doc == Id::NULL && sees!(self, DocLineMarker) {
            doc = self.parse_doc(self.child(&mut builder));
        }

        if sees!(self, PrivateKeyword | PublicKeyword) {
            visibility = Some(self.next());
        }

        if sees!(self, VarKeyword) {
            var_keyword = Some(self.next());
        } else {
            self.syntax_error_end("Variables must start with `var`.");
        }

        if sees!(self, SimpleSymbol(_))
            && !matches!(self.peek_next_significant().kind, EqualSign | Period)
        {
            type_expression = self.parse_type_expression(self.child(&mut builder));
        }

        symbol = self.parse_symbol(self.child(&mut builder));

        if sees!(self, EqualSign) {
            equal_sign = Some(self.next());
            expression = self.parse_expression(self.child(&mut builder));
        }

        if sees!(self, Period) {
            period = Some(self.next());
        } else {
            self.syntax_error_end("Variables must end with a period.");
        }

        self.finalize(
            builder,
            Variable {
                doc,
                visibility,
                var_keyword,
                type_expression,
                symbol,
                equal_sign,
                expression,
                period,
            },
        )
    }

    fn parse_initializer(&mut self, mut doc: Id, mut builder: NodeBuilder) -> Id {
        let mut visibility = None;
        let mut init_keyword = None;
        let message_pattern;
        let mut fat_arrow = None;
        let mut keyword_pairs = vec![];
        let mut period = None;

        if doc == Id::NULL && sees!(self, DocLineMarker) {
            doc = self.parse_doc(self.child(&mut builder));
        }

        if sees!(self, PrivateKeyword | PublicKeyword) {
            visibility = Some(self.next());
        }

        if sees!(self, InitKeyword) {
            init_keyword = Some(self.next());
        } else {
            self.syntax_error_end("Initializers must start with `init`.");
        }

        message_pattern = self.parse_message_pattern(self.child(&mut builder));

        if sees!(self, FatArrow) {
            fat_arrow = Some(self.next());
            keyword_pairs =
                self.parse_keyword_pairs(&mut builder, &Self::parse_initializer_argument);
        }

        if sees!(self, Period) {
            period = Some(self.next());
        } else {
            self.syntax_error_end("Initializers must end with a period.");
        }

        self.finalize(
            builder,
            Initializer {
                doc,
                visibility,
                init_keyword,
                message_pattern,
                fat_arrow,
                keyword_pairs,
                period,
            },
        )
    }

    fn parse_method(&mut self, mut doc: Id, mut builder: NodeBuilder) -> Id {
        let mut visibility = None;
        let mut native_keyword = None;
        let signature;
        let mut method_body = Id::NULL;
        let mut period = None;

        if doc == Id::NULL && sees!(self, DocLineMarker) {
            doc = self.parse_doc(self.child(&mut builder));
        }

        if sees!(self, PrivateKeyword | PublicKeyword) {
            visibility = Some(self.next());
        }

        if sees!(self, NativeKeyword) {
            native_keyword = Some(self.next());
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
                doc,
                visibility,
                native_keyword,
                signature,
                method_body,
                period,
            },
        )
    }

    fn parse_signature(&mut self, mut builder: NodeBuilder) -> Id {
        let mut type_parameter_list = Id::NULL;
        let message_pattern;
        let mut return_type = Id::NULL;

        if sees!(self, OpenAngle) {
            type_parameter_list = self.parse_type_parameter_list(self.child(&mut builder));
        }

        message_pattern = self.parse_message_pattern(self.child(&mut builder));

        if sees!(self, Arrow) {
            return_type = self.parse_return_type(self.child(&mut builder));
        }

        self.finalize(
            builder,
            Signature {
                type_parameter_list,
                message_pattern,
                return_type,
            },
        )
    }

    fn parse_message_pattern(&mut self, builder: NodeBuilder) -> Id {
        if sees!(self, SimpleSymbol(_)) {
            if matches!(self.peek_next_significant().kind, Colon) {
                self.parse_keyword_message_pattern(builder)
            } else {
                self.parse_unary_message_pattern(builder)
            }
        } else if sees!(
            self,
            Dash | Asterisk | Plus | Slash | EqualSign | OpenAngle | CloseAngle
        ) {
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

        if sees!(
            self,
            Dash | Asterisk | Plus | Slash | EqualSign | OpenAngle | CloseAngle
        ) {
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
        if !sees!(self, SimpleSymbol(_)) {
            self.syntax_error("Expected keywords.");
            return Id::NULL;
        }

        let keyword_pairs = self.parse_keyword_pairs(&mut builder, &Self::parse_parameter_pattern);

        self.finalize(builder, KeywordMessagePattern { keyword_pairs })
    }

    fn parse_keyword_pairs<F: Fn(&mut Self, NodeBuilder) -> Id>(
        &mut self,
        builder: &mut NodeBuilder,
        f: &F,
    ) -> Vec<Id> {
        let mut keyword_pairs = vec![];

        while sees!(self, SimpleSymbol(_)) {
            keyword_pairs.push(self.parse_keyword_pair(self.child(builder), f));
        }

        keyword_pairs
    }

    #[inline]
    fn parse_keyword_pair<F: Fn(&mut Self, NodeBuilder) -> Id>(
        &mut self,
        mut builder: NodeBuilder,
        f: &F,
    ) -> Id {
        let keyword;
        let mut colon = None;
        let value;

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
        let mut fat_arrow = None;
        let expression;

        if sees!(self, FatArrow) {
            fat_arrow = Some(self.next());
        } else {
            self.syntax_error("Expected `=>`.");
        }

        expression = self.parse_expression(self.child(&mut builder));

        self.finalize(
            builder,
            MethodBody {
                fat_arrow,
                expression,
            },
        )
    }

    fn parse_return_type(&mut self, mut builder: NodeBuilder) -> Id {
        let mut arrow = None;
        let type_expression;

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

    fn parse_type_expression(&mut self, builder: NodeBuilder) -> Id {
        if sees!(self, SymbolLiteral(_)) {
            self.parse_symbol_type_expression(builder)
        } else if sees!(self, Underscore) {
            self.parse_nothing(builder)
        } else if sees!(self, SelfKeyword) {
            self.parse_self_type_expression(builder)
        } else if sees!(self, SimpleSymbol(_)) {
            self.parse_reference_type_expression(builder)
        } else {
            self.syntax_error("Expected type expression.");
            Id::NULL
        }
    }

    fn parse_self_type_expression(&mut self, builder: NodeBuilder) -> Id {
        if !sees!(self, SelfKeyword) {
            self.syntax_error("Expected self keyword.");
            return Id::NULL;
        }

        let keyword = self.next();

        self.finalize(builder, SelfTypeExpression(keyword))
    }

    fn parse_self_expression(&mut self, builder: NodeBuilder) -> Id {
        if !sees!(self, SelfKeyword) {
            self.syntax_error("Expected self keyword.");
            return Id::NULL;
        }

        let keyword = self.next();

        self.finalize(builder, SelfExpression(keyword))
    }

    fn parse_nothing(&mut self, builder: NodeBuilder) -> Id {
        if !sees!(self, Underscore) {
            self.syntax_error("Expected underscore.");
            return Id::NULL;
        }

        let underscore = self.next();

        self.finalize(builder, Nothing(underscore))
    }

    fn parse_reference_type_expression(&mut self, mut builder: NodeBuilder) -> Id {
        let mut symbol = Id::NULL;
        let mut type_argument_list = Id::NULL;

        if sees!(self, SimpleSymbol(_)) {
            symbol = self.parse_symbol(self.child(&mut builder));
        } else {
            self.syntax_error("Expected type name.");
        }

        if sees!(self, OpenAngle) {
            type_argument_list = self.parse_type_argument_list(self.child(&mut builder));
        }

        self.finalize(
            builder,
            ReferenceTypeExpression {
                symbol,
                type_argument_list,
            },
        )
    }

    fn parse_type_argument_list(&mut self, mut builder: NodeBuilder) -> Id {
        let mut open_angle = None;
        let mut type_expressions = vec![];
        let mut close_angle = None;

        if sees!(self, OpenAngle) {
            open_angle = Some(self.next());
        } else {
            self.syntax_error("Expected type argument list.");
        }

        while !sees!(self, EOF) {
            let before = self.tokens.len();
            type_expressions.push(self.parse_type_expression(self.child(&mut builder)));
            let after = self.tokens.len();

            if before == after {
                self.syntax_error("Unexpected token.");
                self.next();
                break;
            }

            if sees!(self, Comma) {
                self.next();
            } else {
                break;
            }

            if sees!(self, CloseAngle) {
                break;
            }
        }

        if sees!(self, CloseAngle) {
            close_angle = Some(self.next());
        } else {
            self.syntax_error_end("Unterminated type argument list.");
        }

        self.finalize(
            builder,
            TypeArgumentList {
                open_angle,
                type_expressions,
                close_angle,
            },
        )
    }

    fn parse_operator(&mut self, builder: NodeBuilder) -> Id {
        if !sees!(
            self,
            Dash | Asterisk | Plus | Slash | EqualSign | OpenAngle | CloseAngle
        ) {
            return Id::NULL;
        }

        let mut tokens = vec![];
        while sees!(
            self,
            Dash | Asterisk | Plus | Slash | EqualSign | OpenAngle | CloseAngle
        ) {
            tokens.push(self.next());
        }

        self.finalize(builder, Operator(tokens))
    }

    fn parse_parameter_pattern(&mut self, mut builder: NodeBuilder) -> Id {
        let mut type_expression = Id::NULL;
        let mut symbol = Id::NULL;

        if sees!(self, SimpleSymbol(_) | Underscore | SelfKeyword) {
            type_expression = self.parse_type_expression(self.child(&mut builder));
        }

        if sees!(self, SimpleSymbol(_)) {
            if let Colon = self.peek_next_significant().kind {
                // What we're seeing is the next keyword in a keyword pattern
            } else {
                symbol = self.parse_symbol(self.child(&mut builder));
            }
        }

        self.finalize(
            builder,
            ParameterPattern {
                type_expression,
                symbol,
            },
        )
    }

    fn parse_initializer_argument(&mut self, mut builder: NodeBuilder) -> Id {
        if sees!(self, LetKeyword) {
            return self.parse_let_expression(builder);
        }

        let result = self.parse_leaf_expression(self.child(&mut builder));
        let result = self.parse_unary_message_send(self.child(&mut builder), result);
        let result = self.parse_binary_message_send(self.child(&mut builder), result);
        builder.fix_parentage(result)
    }

    fn parse_expression(&mut self, mut builder: NodeBuilder) -> Id {
        if sees!(self, LetKeyword) {
            return self.parse_let_expression(builder);
        }

        let result = self.parse_leaf_expression(self.child(&mut builder));
        let result = self.parse_unary_message_send(self.child(&mut builder), result);
        let result = self.parse_binary_message_send(self.child(&mut builder), result);
        let result = self.parse_keyword_message_send(self.child(&mut builder), result);
        let result = self.parse_cascade_message_send(self.child(&mut builder), result);

        builder.fix_parentage(result)
    }

    fn parse_leaf_expression(&mut self, builder: NodeBuilder) -> Id {
        if sees!(self, OpenParen) {
            return self.parse_tuple_expression(builder);
        }
        if sees!(self, PanicKeyword) {
            return self.parse_panic_expression(builder);
        }
        if sees!(self, SimpleString(_)) {
            return self.parse_string_expression(builder);
        }
        if sees!(self, SimpleCharacter(_)) {
            return self.parse_character_expression(builder);
        }
        if sees!(self, SimpleInteger(_)) {
            return self.parse_integer_expression(builder);
        }
        if sees!(self, SimpleFloat(_)) {
            return self.parse_float_expression(builder);
        }
        if sees!(self, SymbolLiteral(_)) {
            return self.parse_symbol_expression(builder);
        }
        if sees!(self, SelfKeyword) {
            return self.parse_self_expression(builder);
        }
        if sees!(self, SimpleSymbol(_)) {
            return self.parse_reference_expression(builder);
        }
        self.syntax_error_end("Expected expression.");
        Id::NULL
    }

    fn parse_tuple_expression(&mut self, mut builder: NodeBuilder) -> Id {
        let open_paren = Some(self.next());
        let expression;
        let mut close_paren = None;

        expression = self.parse_expression(self.child(&mut builder));

        if sees!(self, CloseParen) {
            close_paren = Some(self.next());
        } else {
            self.syntax_error_end("Expected closing parenthesis.");
        }

        self.finalize(
            builder,
            TupleExpression {
                open_paren,
                expression,
                close_paren,
            },
        )
    }

    fn parse_character_expression(&mut self, builder: NodeBuilder) -> Id {
        if let SimpleCharacter(ref lexeme) = &self.peek().kind {
            let mut contents = vec![];
            let chars = string_to_characters(lexeme.clone());
            let end = chars.len() - 1;
            let mut in_escape = false;
            for (i, c) in chars.into_iter().enumerate() {
                if !in_escape && c == '\\' as u16 {
                    in_escape = true;
                    continue;
                }
                if !in_escape && c == '\'' as u16 && (i == 0 || i == end) {
                    continue;
                }
                in_escape = false;
                contents.push(c);
            }
            let token = self.next();
            if !token.lexeme().ends_with('\'') {
                self.syntax_error_end("Unterminated character literal.");
            }
            if contents.len() == 0 {
                self.syntax_error_end("Empty character literal.");
            }
            self.finalize(
                builder,
                CharacterExpression(token, contents.into_iter().next()),
            )
        } else {
            self.syntax_error("Expected character.");
            Id::NULL
        }
    }

    fn parse_string_expression(&mut self, builder: NodeBuilder) -> Id {
        if let SimpleString(ref lexeme) = &self.peek().kind {
            let mut contents = vec![];
            let chars = string_to_characters(lexeme.clone());
            let end = chars.len() - 1;
            let mut in_escape = false;
            for (i, c) in chars.into_iter().enumerate() {
                if !in_escape && c == '\\' as u16 {
                    in_escape = true;
                    continue;
                }
                if !in_escape && c == '"' as u16 && (i == 0 || i == end) {
                    continue;
                }
                in_escape = false;
                contents.push(c);
            }
            let token = self.next();
            if !token.lexeme().ends_with('"') {
                self.syntax_error_end("Unterminated string.");
            }
            self.finalize(
                builder,
                StringExpression(token, characters_to_string(contents.into_iter())),
            )
        } else {
            self.syntax_error("Expected string.");
            Id::NULL
        }
    }

    fn parse_integer_expression(&mut self, builder: NodeBuilder) -> Id {
        if let SimpleInteger(ref lexeme) = &self.peek().kind {
            let (base, rest) = Self::split_number_base(lexeme);

            let int = BigInt::parse_bytes(rest.as_bytes(), base).unwrap();
            let token = self.next();

            self.finalize(builder, IntegerExpression(token, int))
        } else {
            self.syntax_error("Expected integer.");
            Id::NULL
        }
    }

    fn split_number_base(lexeme: &str) -> (u32, &str) {
        let base_split = lexeme.split("#").collect::<Vec<_>>();

        if base_split.len() == 2 {
            (
                u32::from_str_radix(base_split[0], 10).unwrap(),
                &base_split[1],
            )
        } else {
            (10, base_split[0])
        }
    }

    fn parse_float_expression(&mut self, builder: NodeBuilder) -> Id {
        if let SimpleFloat(ref lexeme) = &self.peek().kind {
            let (base, rest) = Self::split_number_base(lexeme);

            let split = rest.split(".").collect::<Vec<_>>();
            let precision = split[1].len();
            let as_int = format!("{}{}", split[0], split[1]);
            let int = BigUint::parse_bytes(as_int.as_bytes(), base).unwrap();
            let fraction = BigFraction::new(int, BigUint::from(base).pow(precision));

            let token = self.next();
            self.finalize(builder, FloatExpression(token, fraction))
        } else {
            self.syntax_error("Expected float.");
            Id::NULL
        }
    }

    fn parse_symbol_expression(&mut self, builder: NodeBuilder) -> Id {
        if let SymbolLiteral(ref lexeme) = &self.peek().kind {
            let symbol = lexeme[1..].to_string();
            let token = self.next();
            self.finalize(builder, SymbolExpression(token, symbol))
        } else {
            self.syntax_error("Expected symbol.");
            Id::NULL
        }
    }

    fn parse_symbol_type_expression(&mut self, builder: NodeBuilder) -> Id {
        if let SymbolLiteral(ref lexeme) = &self.peek().kind {
            let symbol = lexeme[1..].to_string();
            let token = self.next();
            self.finalize(builder, SymbolTypeExpression(token, symbol))
        } else {
            self.syntax_error("Expected symbol.");
            Id::NULL
        }
    }

    fn parse_panic_expression(&mut self, mut builder: NodeBuilder) -> Id {
        let panic_keyword = self.next();
        let expression = self.parse_expression(self.child(&mut builder));

        self.finalize(
            builder,
            PanicExpression {
                panic_keyword,
                expression,
            },
        )
    }

    fn parse_unary_message_send(&mut self, mut builder: NodeBuilder, receiver: Id) -> Id {
        if sees!(self, SimpleSymbol(_)) {
            if self.peek_next_significant().kind != Colon {
                let message = {
                    let mut builder = self.child(&mut builder);

                    let symbol = self.parse_symbol(self.child(&mut builder));

                    self.finalize(builder, UnaryMessage { symbol })
                };

                let result = self.finalize(
                    self.child(&mut builder),
                    MessageSendExpression {
                        expression: receiver,
                        message,
                    },
                );
                return self.parse_unary_message_send(self.child(&mut builder), result);
            }
        }
        return receiver;
    }

    fn parse_binary_message_send(&mut self, mut builder: NodeBuilder, receiver: Id) -> Id {
        if sees!(
            self,
            Dash | Asterisk | Plus | Slash | EqualSign | OpenAngle | CloseAngle
        ) {
            let message = {
                let mut builder = self.child(&mut builder);

                let operator = self.parse_operator(self.child(&mut builder));
                let result = self.parse_leaf_expression(self.child(&mut builder));
                let expression = self.parse_unary_message_send(self.child(&mut builder), result);

                self.finalize(
                    builder,
                    BinaryMessage {
                        operator,
                        expression,
                    },
                )
            };

            let result = self.finalize(
                self.child(&mut builder),
                MessageSendExpression {
                    expression: receiver,
                    message,
                },
            );
            let result = self.parse_unary_message_send(self.child(&mut builder), result);
            return self.parse_binary_message_send(self.child(&mut builder), result);
        }
        return receiver;
    }

    fn parse_keyword_argument(&mut self, mut builder: NodeBuilder) -> Id {
        let result = self.parse_leaf_expression(self.child(&mut builder));
        let result = self.parse_unary_message_send(self.child(&mut builder), result);
        self.parse_binary_message_send(self.child(&mut builder), result)
    }

    fn parse_keyword_message_send(&mut self, mut builder: NodeBuilder, receiver: Id) -> Id {
        if sees!(self, SimpleSymbol(_)) {
            let message = {
                let mut builder = self.child(&mut builder);

                let keyword_pairs =
                    self.parse_keyword_pairs(&mut builder, &Self::parse_keyword_argument);

                self.finalize(builder, KeywordMessage { keyword_pairs })
            };

            return self.finalize(
                builder,
                MessageSendExpression {
                    expression: receiver,
                    message,
                },
            );
        }

        return receiver;
    }

    fn parse_cascade_message_send(&mut self, mut builder: NodeBuilder, receiver: Id) -> Id {
        if sees!(self, SemiColon) {
            let result = {
                let builder = self.child(&mut builder);

                let semi_colon = Some(self.next());

                self.finalize(
                    builder,
                    CascadeExpression {
                        expression: receiver,
                        semi_colon,
                    },
                )
            };

            let result = self.parse_unary_message_send(self.child(&mut builder), result);
            let result = self.parse_binary_message_send(self.child(&mut builder), result);
            let result = self.parse_keyword_message_send(self.child(&mut builder), result);
            let result = self.parse_cascade_message_send(self.child(&mut builder), result);

            return result;
        }
        return receiver;
    }

    fn parse_reference_expression(&mut self, mut builder: NodeBuilder) -> Id {
        let symbol = self.parse_symbol(self.child(&mut builder));

        self.finalize(builder, ReferenceExpression { symbol })
    }

    fn parse_let_expression(&mut self, mut builder: NodeBuilder) -> Id {
        let mut let_binding = Id::NULL;

        if sees!(self, LetKeyword) {
            let_binding = self.parse_let_binding(self.child(&mut builder));
        } else {
            self.syntax_error("Expected let binding.");
        }

        let expression = self.parse_expression(self.child(&mut builder));

        self.finalize(
            builder,
            LetExpression {
                let_binding,
                expression,
            },
        )
    }

    fn parse_let_binding(&mut self, mut builder: NodeBuilder) -> Id {
        let mut let_keyword = None;
        let mut type_expression = Id::NULL;
        let mut symbol = Id::NULL;
        let mut equal_sign = None;
        let mut period = None;

        if sees!(self, LetKeyword) {
            let_keyword = Some(self.next());
        } else {
            self.syntax_error("Binding must start with `let`.");
        }

        if self.tokens.len() >= 2
            && !matches!(
                (&self.peek().kind, &self.peek_next_significant().kind),
                (SimpleSymbol(_), EqualSign)
            )
        {
            type_expression = self.parse_type_expression(self.child(&mut builder));
        }

        if sees!(self, SimpleSymbol(_)) {
            symbol = self.parse_symbol(self.child(&mut builder));
        } else {
            self.syntax_error_end("Binding must have a name.");
        }

        if sees!(self, EqualSign) {
            equal_sign = Some(self.next());
        } else {
            self.syntax_error_end("Expected equal sign.");
        }

        let expression = self.parse_expression(self.child(&mut builder));

        if sees!(self, Period) {
            period = Some(self.next());
        } else {
            self.syntax_error_end("Binding must end with period.");
        }

        self.finalize(
            builder,
            LetBinding {
                let_keyword,
                type_expression,
                symbol,
                equal_sign,
                expression,
                period,
            },
        )
    }
}
