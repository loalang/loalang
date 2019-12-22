use crate::fmt::{Display, Formatter as F, Result};
use crate::syntax::*;
use crate::*;

pub struct Formatter<'a> {
    tree: &'a Tree,
    indentation: usize,
    indent: &'a str,
    is_in_doc: bool,
}

impl<'a> Formatter<'a> {
    pub fn new(tree: &'a Tree, indent: &'a str) -> Formatter<'a> {
        Formatter {
            tree,
            indentation: 0,
            indent,
            is_in_doc: false,
        }
    }

    pub fn format(tree: &'a Tree, indent: &'a str) -> String {
        let display = FormatterDisplay { tree, indent };
        format!("{}", display)
    }

    fn write_tree(&mut self, f: &mut F) -> Result {
        self.tree
            .root()
            .map(|root| {
                self.write_node(f, root)?;
                let mut comments = root.insignificant_tokens_after(&self.tree);
                comments.retain(|c| match c.kind {
                    TokenKind::LineComment(_) => true,
                    _ => false,
                });
                if comments.len() > 0 {
                    self.break_line(f)?;
                    self.write_comments(f, comments)?;
                }
                Ok(())
            })
            .unwrap_or(Ok(()))
    }

    fn indent(&mut self) {
        self.indentation += 1;
    }

    fn outdent(&mut self) {
        self.indentation -= 1;
    }

    fn write_token(&mut self, f: &mut F, token: &Token) -> Result {
        self.write_comments(f, token.before.clone())?;
        write!(f, "{}", token.lexeme())
    }

    fn write_comments(&mut self, f: &mut F, tokens: Vec<Token>) -> Result {
        for token in tokens {
            if let TokenKind::LineComment(_) = token.kind {
                write!(f, "{}", token.lexeme())?;
                self.break_line(f)?;
            }
        }
        Ok(())
    }

    fn write_token_or(&mut self, f: &mut F, token: &Option<Token>, fallback: &str) -> Result {
        match token {
            Some(token) => self.write_token(f, token),
            None => write!(f, "{}", fallback),
        }
    }

    fn write_child(&mut self, f: &mut F, id: &Id) -> Result {
        self.tree
            .borrow(*id)
            .map(|root| self.write_node(f, root))
            .unwrap_or(Ok(()))
    }

    fn space(&self, f: &mut F) -> Result {
        write!(f, " ")
    }

    fn break_line(&self, f: &mut F) -> Result {
        write!(f, "\n")?;
        for _ in 0..self.indentation {
            write!(f, "{}", self.indent)?;
        }
        if self.is_in_doc {
            write!(f, "/// ")?;
        }
        Ok(())
    }

    fn write_node(&mut self, f: &mut F, node: &Node) -> Result {
        match &node.kind {
            Module {
                namespace_directive,
                import_directives,
                module_declarations,
            } => {
                self.write_child(f, namespace_directive)?;
                self.break_line(f)?;
                if !import_directives.is_empty() {
                    self.break_line(f)?;
                    for import in import_directives.iter() {
                        self.write_child(f, import)?;
                        self.break_line(f)?;
                    }
                }
                for declaration in module_declarations.iter() {
                    self.break_line(f)?;
                    self.write_child(f, declaration)?;
                    self.break_line(f)?;
                }
                Ok(())
            }
            REPLLine { statements } => {
                for (i, statement) in statements.iter().enumerate() {
                    if i > 0 {
                        self.break_line(f)?;
                    }
                    self.write_child(f, statement)?;
                }
                Ok(())
            }
            REPLDirective {
                colon,
                symbol,
                expression,
                period,
            } => {
                self.write_token(f, colon)?;
                self.write_child(f, symbol)?;
                self.space(f)?;
                self.write_child(f, expression)?;
                self.write_token_or(f, period, ".")
            }
            REPLExpression { expression, period } => {
                self.write_child(f, expression)?;
                self.write_token_or(f, period, ".")
            }
            Exported(doc, export_keyword, declaration) => {
                self.write_child(f, doc)?;
                self.write_token(f, export_keyword)?;
                self.space(f)?;
                self.write_child(f, declaration)
            }
            NamespaceDirective {
                namespace_keyword,
                qualified_symbol,
                period,
            } => {
                self.write_token_or(f, namespace_keyword, "namespace")?;
                self.space(f)?;
                self.write_child(f, qualified_symbol)?;
                self.write_token_or(f, period, ".")
            }
            ImportDirective {
                import_keyword,
                qualified_symbol,
                as_keyword,
                symbol,
                period,
            } => {
                self.write_token_or(f, import_keyword, "import")?;
                self.space(f)?;
                self.write_child(f, qualified_symbol)?;
                if !symbol.is_null() {
                    self.space(f)?;
                    self.write_token_or(f, as_keyword, "as")?;
                    self.space(f)?;
                    self.write_child(f, symbol)?;
                }
                self.write_token_or(f, period, ".")
            }
            QualifiedSymbol { symbols } => {
                for (i, symbol) in symbols.iter().enumerate() {
                    if i > 0 {
                        write!(f, "/")?;
                    }
                    self.write_child(f, symbol)?;
                }
                Ok(())
            }
            Symbol(token) => self.write_token(f, token),
            Class {
                doc,
                partial_keyword,
                class_keyword,
                symbol,
                type_parameter_list,
                class_body,
                period,
            } => {
                self.write_child(f, doc)?;
                if let Some(partial_keyword) = partial_keyword {
                    self.write_token(f, partial_keyword)?;
                    self.space(f)?;
                }
                self.write_token_or(f, class_keyword, "class")?;
                self.space(f)?;
                self.write_child(f, symbol)?;
                self.write_child(f, type_parameter_list)?;
                if !class_body.is_null() {
                    self.space(f)?;
                    self.write_child(f, class_body)?;
                } else {
                    self.write_token_or(f, period, ".")?;
                }
                Ok(())
            }
            TypeParameterList {
                open_angle,
                type_parameters,
                close_angle,
            } => {
                self.write_token_or(f, open_angle, "<")?;
                for (i, param) in type_parameters.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    self.write_child(f, param)?;
                }
                self.write_token_or(f, close_angle, ">")
            }
            TypeParameter {
                symbol,
                variance_keyword,
            } => {
                self.write_child(f, symbol)?;
                if let Some(variance_keyword) = variance_keyword {
                    self.space(f)?;
                    self.write_token(f, variance_keyword)?;
                }
                Ok(())
            }
            ClassBody {
                open_curly,
                class_members,
                close_curly,
            } => {
                self.write_token_or(f, open_curly, "{")?;
                self.indent();
                self.break_line(f)?;

                for (i, member) in class_members.iter().enumerate() {
                    if i > 0 {
                        self.break_line(f)?;
                        self.break_line(f)?;
                    }
                    self.write_child(f, member)?;
                }

                self.outdent();
                self.break_line(f)?;
                self.write_token_or(f, close_curly, "}")
            }
            Method {
                visibility,
                native_keyword,
                signature,
                method_body,
                period,
            } => {
                self.write_token_or(f, visibility, "private")?;
                self.space(f)?;
                if native_keyword.is_some() {
                    self.write_token_or(f, native_keyword, "native")?;
                    self.space(f)?;
                }
                self.write_child(f, signature)?;
                if !method_body.is_null() {
                    self.space(f)?;
                    self.write_child(f, method_body)?;
                }
                self.write_token_or(f, period, ".")
            }
            IsDirective {
                is_keyword,
                type_expression,
                period,
            } => {
                self.write_token(f, is_keyword)?;
                self.space(f)?;
                self.write_child(f, type_expression)?;
                self.write_token_or(f, period, ".")
            }
            Signature {
                type_parameter_list,
                message_pattern,
                return_type,
            } => {
                if !type_parameter_list.is_null() {
                    self.write_child(f, type_parameter_list)?;
                    self.space(f)?;
                }
                self.write_child(f, message_pattern)?;
                if !return_type.is_null() {
                    self.space(f)?;
                    self.write_child(f, return_type)?;
                }
                Ok(())
            }
            UnaryMessagePattern { symbol } => self.write_child(f, symbol),
            BinaryMessagePattern {
                operator,
                parameter_pattern,
            } => {
                self.write_child(f, operator)?;
                self.space(f)?;
                self.write_child(f, parameter_pattern)
            }
            Operator(tokens) => {
                for token in tokens.iter() {
                    self.write_token(f, token)?;
                }
                Ok(())
            }
            KeywordMessagePattern { keyword_pairs } => {
                for (i, pair) in keyword_pairs.iter().enumerate() {
                    if i > 0 {
                        self.space(f)?;
                    }
                    self.write_child(f, pair)?;
                }
                Ok(())
            }
            KeywordPair {
                keyword,
                colon,
                value,
            } => {
                self.write_child(f, keyword)?;
                self.write_token_or(f, colon, ":")?;
                self.space(f)?;
                self.write_child(f, value)
            }
            ReturnType {
                arrow,
                type_expression,
            } => {
                self.write_token_or(f, arrow, "->")?;
                self.space(f)?;
                self.write_child(f, type_expression)
            }
            ParameterPattern {
                type_expression,
                symbol,
            } => {
                self.write_child(f, type_expression)?;
                if !symbol.is_null() {
                    self.space(f)?;
                    self.write_child(f, symbol)?;
                }
                Ok(())
            }
            ReferenceTypeExpression {
                symbol,
                type_argument_list,
            } => {
                self.write_child(f, symbol)?;
                self.write_child(f, type_argument_list)
            }
            SelfTypeExpression(self_keyword) => self.write_token(f, self_keyword),
            Nothing(underscore) => self.write_token(f, underscore),
            SymbolTypeExpression(literal, _) => self.write_token(f, literal),
            TypeArgumentList {
                open_angle,
                type_expressions,
                close_angle,
            } => {
                self.write_token_or(f, open_angle, "<")?;
                for (i, arg) in type_expressions.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    self.write_child(f, arg)?;
                }
                self.write_token_or(f, close_angle, ">")
            }
            MethodBody {
                fat_arrow,
                expression,
            } => {
                self.write_token_or(f, fat_arrow, "=>")?;
                self.indent();
                self.break_line(f)?;
                self.write_child(f, expression)?;
                self.outdent();
                Ok(())
            }
            ReferenceExpression { symbol } => self.write_child(f, symbol),
            SelfExpression(self_keyword) => self.write_token(f, self_keyword),
            StringExpression(literal, _) => self.write_token(f, literal),
            CharacterExpression(literal, _) => self.write_token(f, literal),
            IntegerExpression(literal, _) => self.write_token(f, literal),
            FloatExpression(literal, _) => self.write_token(f, literal),
            SymbolExpression(literal, _) => self.write_token(f, literal),
            MessageSendExpression {
                expression,
                message,
            } => {
                self.write_child(f, expression)?;
                self.space(f)?;
                self.write_child(f, message)
            }
            UnaryMessage { symbol } => self.write_child(f, symbol),
            BinaryMessage {
                operator,
                expression,
            } => {
                self.write_child(f, operator)?;
                self.space(f)?;
                self.write_child(f, expression)
            }
            KeywordMessage { keyword_pairs } => {
                for (i, pair) in keyword_pairs.iter().enumerate() {
                    if i > 0 {
                        self.space(f)?;
                    }
                    self.write_child(f, pair)?;
                }
                Ok(())
            }
            LetExpression {
                let_binding,
                expression,
            } => {
                self.write_child(f, let_binding)?;
                self.break_line(f)?;
                self.write_child(f, expression)
            }
            LetBinding {
                let_keyword,
                type_expression,
                symbol,
                equal_sign,
                expression,
                period,
            } => {
                self.write_token_or(f, let_keyword, "let")?;
                self.space(f)?;
                if !type_expression.is_null() {
                    self.write_child(f, type_expression)?;
                    self.space(f)?;
                }
                self.write_child(f, symbol)?;
                self.space(f)?;
                self.write_token_or(f, equal_sign, "=")?;
                self.space(f)?;
                self.write_child(f, expression)?;
                self.write_token_or(f, period, ".")
            }

            Doc {
                doc_line_marker,
                blocks,
            } => {
                self.write_token(f, doc_line_marker)?;
                self.space(f)?;
                self.is_in_doc = true;
                for (i, block) in blocks.iter().enumerate() {
                    if i > 0 {
                        self.break_line(f)?;
                        self.break_line(f)?;
                    }
                    self.write_child(f, block)?;
                }
                self.is_in_doc = false;
                self.break_line(f)
            }

            DocParagraphBlock { elements } => {
                for element in elements.iter() {
                    self.write_child(f, element)?;
                }
                Ok(())
            }

            DocTextElement(ref tokens) => {
                for token in tokens.iter() {
                    self.write_token(f, token)?;
                }
                Ok(())
            }

            DocItalicElement(ref open, ref tokens, ref close) => {
                self.write_token(f, open)?;
                for token in tokens.iter() {
                    self.write_token(f, token)?;
                }
                self.write_token(f, close)
            }

            DocBoldElement(ref open, ref tokens, ref close) => {
                self.write_token(f, open)?;
                for token in tokens.iter() {
                    self.write_token(f, token)?;
                }
                self.write_token(f, close)
            }
        }
    }
}

struct FormatterDisplay<'a> {
    tree: &'a Tree,
    indent: &'a str,
}

impl<'a> Display for FormatterDisplay<'a> {
    fn fmt(&self, f: &mut F) -> Result {
        Formatter::new(self.tree, self.indent).write_tree(f)
    }
}

impl Display for Tree {
    fn fmt(&self, f: &mut F) -> Result {
        if f.alternate() {
            Formatter::new(self, "  ").write_tree(f)
        } else {
            write!(f, "{}", self.source.code)
        }
    }
}
