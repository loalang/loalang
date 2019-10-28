use crate::syntax::*;
use crate::*;

#[derive(Clone)]
pub struct Node {
    pub id: Id,
    pub parent_id: Option<Id>,
    pub span: Span,
    pub kind: NodeKind,
}

impl Node {
    pub fn children(&self) -> Vec<Id> {
        self.kind.children()
    }

    pub fn leaves(&self) -> Vec<&Token> {
        self.kind.leaves()
    }

    pub fn is_symbol(&self) -> bool {
        match self.kind {
            Symbol(_) => true,
            _ => false,
        }
    }

    pub fn is_scope_root(&self) -> bool {
        match self.kind {
            Module { .. } | ClassBody { .. } | Method { .. } => true,
            _ => false,
        }
    }

    pub fn is_import_directive(&self) -> bool {
        match self.kind {
            ImportDirective { .. } => true,
            _ => false,
        }
    }

    pub fn is_qualified_symbol(&self) -> bool {
        match self.kind {
            QualifiedSymbol { .. } => true,
            _ => false,
        }
    }

    pub fn is_declaration(&self) -> bool {
        match self.kind {
            Class { .. } | ParameterPattern { .. } => true,
            _ => false,
        }
    }

    pub fn is_reference(&self) -> bool {
        match self.kind {
            ReferenceTypeExpression { .. } | ReferenceExpression { .. } => true,
            _ => false,
        }
    }

    pub fn is_expression(&self) -> bool {
        match self.kind {
            ReferenceExpression { .. } | MessageSendExpression { .. } => true,
            _ => false,
        }
    }

    pub fn is_message(&self) -> bool {
        match self.kind {
            UnaryMessage { .. } | BinaryMessage { .. } | KeywordMessage { .. } => true,
            _ => false,
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {}: {:?}", self.id, self.span, self.kind)
    }
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    /// ```bnf
    /// Module ::=
    ///   NamespaceDirective
    ///   ImportDirective*
    ///   ModuleDeclaration*
    /// ```
    Module {
        namespace_directive: Id,
        import_directives: Vec<Id>,
        module_declarations: Vec<Id>,
    },

    /// ```bnf
    /// ModuleDeclaration ::=
    ///   Declaration | EXPORT_KEYWORD Declaration
    /// ```
    Exported(Token, Id),

    /// ```bnf
    /// NamespaceDirective ::=
    ///   NAMESPACE_KEYWORD
    ///   QualifiedSymbol
    ///   PERIOD
    /// ```
    NamespaceDirective {
        namespace_keyword: Option<Token>,
        qualified_symbol: Id,
        period: Option<Token>,
    },

    /// ```bnf
    /// ImportDirective ::=
    ///   IMPORT_KEYWORD
    ///   QualifiedSymbol
    ///   (
    ///     AS_KEYWORD
    ///     Symbol
    ///   )?
    ///   PERIOD
    /// ```
    ImportDirective {
        import_keyword: Option<Token>,
        qualified_symbol: Id,
        as_keyword: Option<Token>,
        symbol: Id,
        period: Option<Token>,
    },

    /// ```bnf
    /// QualifiedSymbol ::=
    ///   Symbol
    ///   (
    ///     SLASH
    ///     Symbol
    ///   )*
    /// ```
    QualifiedSymbol { symbols: Vec<Id> },

    /// ```bnf
    /// Symbol ::=
    ///   SIMPLE_SYMBOL
    /// ```
    Symbol(Token),

    /// ```bnf
    /// Declaration ::=
    ///   Class
    /// ```

    /// ```bnf
    /// Class ::=
    ///   CLASS_KEYWORD
    ///   Symbol
    ///   (ClassBody | PERIOD)
    /// ```
    Class {
        class_keyword: Option<Token>,
        symbol: Id,
        class_body: Id,
        period: Option<Token>,
    },

    /// ```bnf
    /// ClassBody ::=
    ///   OPEN_CURLY
    ///   ClassMember*
    ///   CLOSE_CURLY
    /// ```
    ClassBody {
        open_curly: Option<Token>,
        class_members: Vec<Id>,
        close_curly: Option<Token>,
    },

    /// ```bnf
    /// ClassMember ::=
    ///   Method
    /// ```

    /// ```bnf
    /// Method ::=
    ///   (PUBLIC_KEYWORD | PRIVATE_KEYWORD)?
    ///   Signature
    ///   MethodBody?
    ///   PERIOD
    /// ```
    Method {
        visibility: Option<Token>,
        signature: Id,
        method_body: Id,
        period: Option<Token>,
    },

    /// ```bnf
    /// Signature ::=
    ///   MessagePattern
    ///   ReturnType?
    /// ```
    Signature {
        message_pattern: Id,
        return_type: Id,
    },

    /// ```bnf
    /// MessagePattern ::=
    ///   UnaryMessagePattern |
    ///   BinaryMessagePattern |
    ///   KeywordMessagePattern
    /// ```

    /// ```bnf
    /// UnaryMessagePattern ::=
    ///   Symbol
    /// ```
    UnaryMessagePattern { symbol: Id },

    /// ```bnf
    /// BinaryMessagePattern ::=
    ///   Operator
    ///   ParameterPattern
    /// ```
    BinaryMessagePattern { operator: Id, parameter_pattern: Id },

    /// ```bnf
    /// Operator ::=
    ///   (PLUS | SLASH | EQUAL_SIGN | OPEN_ANGLE | CLOSE_ANGLE)+
    /// ```
    Operator(Token),

    /// ```bnf
    /// KeywordMessagePattern ::=
    ///   KeywordPair<ParameterPattern>+
    /// ```
    KeywordMessagePattern { keyword_pairs: Vec<Id> },

    /// ```bnf
    /// KeywordPair<N> ::=
    ///   Symbol
    ///   COLON
    ///   N
    /// ```
    KeywordPair {
        keyword: Id,
        colon: Option<Token>,
        value: Id,
    },

    /// ```bnf
    /// ReturnType ::=
    ///   ARROW
    ///   TypeExpression
    /// ```
    ReturnType {
        arrow: Option<Token>,
        type_expression: Id,
    },

    /// ```bnf
    /// ParameterPattern ::=
    ///   TypeExpression |
    ///   Symbol |
    ///   (
    ///     TypeExpression
    ///     Symbol
    ///   )
    /// ```
    ParameterPattern { type_expression: Id, symbol: Id },

    /// ```bnf
    /// TypeExpression ::=
    ///   ReferenceTypeExpression
    /// ```

    /// ```bnf
    /// ReferenceTypeExpression ::=
    ///   Symbol
    /// ```
    ReferenceTypeExpression { symbol: Id },

    /// ```bnf
    /// MethodBody ::=
    ///   FAT_ARROW
    ///   Expression
    /// ```
    MethodBody {
        fat_arrow: Option<Token>,
        expression: Id,
    },

    /// ```bnf
    /// Expression ::=
    ///   ReferenceExpression |
    ///   MessageSendExpression
    /// ```

    /// ```bnf
    /// ReferenceExpression ::=
    ///   Symbol
    /// ```
    ReferenceExpression { symbol: Id },

    /// ```bnf
    /// MessageSendExpression ::=
    ///   Expression
    ///   Message
    /// ```
    MessageSendExpression { expression: Id, message: Id },

    /// ```bnf
    /// Message ::=
    ///   UnaryMessage |
    ///   BinaryMessage |
    ///   KeywordMessage
    /// ```

    /// ```bnf
    /// UnaryMessage ::=
    ///   Symbol
    /// ```
    UnaryMessage { symbol: Id },

    /// ```bnf
    /// BinaryMessage ::=
    ///   Operator
    ///   Expression
    /// ```
    BinaryMessage { operator: Id, expression: Id },

    /// ```bnf
    /// KeywordMessage ::=
    ///   KeywordPair<Expression>+
    /// ```
    KeywordMessage { keyword_pairs: Vec<Id> },
}

pub use NodeKind::*;

impl NodeKind {
    pub fn leaves(&self) -> Vec<&Token> {
        let option_tokens: Vec<Option<&Token>> = match self {
            Exported(ref token, _) => vec![Some(token)],

            NamespaceDirective {
                ref namespace_keyword,
                ref period,
                ..
            } => vec![namespace_keyword.as_ref(), period.as_ref()],

            ImportDirective {
                ref import_keyword,
                ref as_keyword,
                ref period,
                ..
            } => vec![
                import_keyword.as_ref(),
                as_keyword.as_ref(),
                period.as_ref(),
            ],

            Symbol(ref token) => vec![Some(token)],

            Class {
                ref class_keyword,
                ref period,
                ..
            } => vec![class_keyword.as_ref(), period.as_ref()],

            ClassBody {
                ref open_curly,
                ref close_curly,
                ..
            } => vec![open_curly.as_ref(), close_curly.as_ref()],

            Method {
                ref visibility,
                ref period,
                ..
            } => vec![visibility.as_ref(), period.as_ref()],

            Operator(ref token) => vec![Some(token)],

            KeywordPair { ref colon, .. } => vec![colon.as_ref()],

            ReturnType { ref arrow, .. } => vec![arrow.as_ref()],

            MethodBody { ref fat_arrow, .. } => vec![fat_arrow.as_ref()],

            _ => vec![],
        };

        option_tokens.into_iter().filter_map(|t| t).collect()
    }

    pub fn children(&self) -> Vec<Id> {
        let mut children = vec![];

        match self {
            Module {
                namespace_directive,
                import_directives,
                module_declarations,
            } => {
                children.push(namespace_directive);
                children.extend(import_directives);
                children.extend(module_declarations);
            }
            Exported(_, declaration) => {
                children.push(declaration);
            }
            NamespaceDirective {
                qualified_symbol, ..
            } => {
                children.push(qualified_symbol);
            }
            ImportDirective {
                qualified_symbol,
                symbol,
                ..
            } => {
                children.push(qualified_symbol);
                children.push(symbol);
            }
            QualifiedSymbol { symbols } => {
                children.extend(symbols);
            }
            Symbol(_) => {}
            Class {
                symbol, class_body, ..
            } => {
                children.push(symbol);
                children.push(class_body);
            }
            ClassBody { class_members, .. } => {
                children.extend(class_members);
            }
            Method {
                signature,
                method_body,
                ..
            } => {
                children.push(signature);
                children.push(method_body);
            }
            Signature {
                message_pattern,
                return_type,
            } => {
                children.push(message_pattern);
                children.push(return_type);
            }
            UnaryMessagePattern { symbol } => {
                children.push(symbol);
            }
            BinaryMessagePattern {
                operator,
                parameter_pattern,
            } => {
                children.push(operator);
                children.push(parameter_pattern);
            }
            Operator(_) => {}
            KeywordMessagePattern { keyword_pairs } => {
                children.extend(keyword_pairs);
            }
            KeywordPair { keyword, value, .. } => {
                children.push(keyword);
                children.push(value);
            }
            ReturnType {
                type_expression, ..
            } => {
                children.push(type_expression);
            }
            ParameterPattern {
                type_expression,
                symbol,
            } => {
                children.push(type_expression);
                children.push(symbol);
            }
            ReferenceTypeExpression { symbol } => {
                children.push(symbol);
            }
            MethodBody { expression, .. } => {
                children.push(expression);
            }
            ReferenceExpression { symbol } => {
                children.push(symbol);
            }
            MessageSendExpression {
                expression,
                message,
            } => {
                children.push(expression);
                children.push(message);
            }
            UnaryMessage { symbol } => {
                children.push(symbol);
            }
            BinaryMessage {
                operator,
                expression,
            } => {
                children.push(operator);
                children.push(expression);
            }
            KeywordMessage { keyword_pairs } => {
                children.extend(keyword_pairs);
            }
        }

        children
            .into_iter()
            .cloned()
            .filter(|i| *i != Id::NULL)
            .collect::<Vec<_>>()
    }
}

pub struct NodeBuilder<'a> {
    tree: &'a mut Tree,
    start: Location,
    pub id: Id,
    parent_id: Option<Id>,
}

impl<'a> NodeBuilder<'a> {
    pub fn new(tree: &'a mut Tree, start: Location) -> NodeBuilder<'a> {
        NodeBuilder {
            tree,
            start,
            id: Id::new(),
            parent_id: None,
        }
    }

    pub fn child(&mut self, start: Location) -> NodeBuilder {
        NodeBuilder {
            tree: &mut self.tree,
            start,
            id: Id::new(),
            parent_id: Some(self.id),
        }
    }

    pub fn finalize(self, end: Location, kind: NodeKind) -> Id {
        self.tree.add(Node {
            id: self.id,
            span: Span::new(self.start, end),
            parent_id: self.parent_id,
            kind,
        });
        self.id
    }

    pub fn fix_parentage(mut self, id: Id) -> Id {
        fix_parentage(&mut self.tree, id, self.parent_id);
        id
    }
}

fn fix_parentage(tree: &mut Tree, id: Id, parent_id: Option<Id>) {
    if let Some(node) = tree.get_mut(id) {
        node.parent_id = parent_id;
        for child in node.children() {
            fix_parentage(tree, child, Some(id));
        }
    }
}
