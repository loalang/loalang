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

    pub fn insignificant_tokens_after(&self, tree: &Tree) -> Vec<Token> {
        self.last_leaf(tree)
            .map(|leaf| leaf.after.clone())
            .unwrap_or(vec![])
    }

    fn last_leaf<'a>(&'a self, tree: &'a Tree) -> Option<&'a Token> {
        let mut candidates = vec![];

        candidates.extend(self.leaves().last().into_iter());
        candidates.extend(
            self.children()
                .last()
                .and_then(|c| tree.borrow(*c))
                .and_then(|n| n.last_leaf(tree))
                .into_iter(),
        );

        candidates.sort_by(|a, b| a.span.end.cmp(&b.span.end));

        candidates.last().map(|t| *t)
    }

    pub fn is_symbol(&self) -> bool {
        match self.kind {
            Symbol(_) => true,
            _ => false,
        }
    }

    pub fn is_operator(&self) -> bool {
        match self.kind {
            Operator(_) => true,
            _ => false,
        }
    }

    pub fn is_repl_line(&self) -> bool {
        match self.kind {
            REPLLine { .. } => true,
            _ => false,
        }
    }

    pub fn is_scope_root(&self) -> bool {
        match self.kind {
            REPLLine { .. } | Module { .. } | Class { .. } | Method { .. } | LetBinding { .. } => {
                true
            }
            _ => false,
        }
    }

    pub fn is_let_binding(&self) -> bool {
        match self.kind {
            LetBinding { .. } => true,
            _ => false,
        }
    }

    pub fn is_class(&self) -> bool {
        match self.kind {
            Class { .. } => true,
            _ => false,
        }
    }

    pub fn is_method(&self) -> bool {
        match self.kind {
            Method { .. } => true,
            _ => false,
        }
    }

    pub fn is_message_send(&self) -> bool {
        match self.kind {
            MessageSendExpression { .. } => true,
            _ => false,
        }
    }

    pub fn is_import_directive(&self) -> bool {
        match self.kind {
            ImportDirective { .. } => true,
            _ => false,
        }
    }

    pub fn is_is_directive(&self) -> bool {
        match self.kind {
            IsDirective { .. } => true,
            _ => false,
        }
    }

    pub fn is_qualified_symbol(&self) -> bool {
        match self.kind {
            QualifiedSymbol { .. } => true,
            _ => false,
        }
    }

    pub fn is_number_literal(&self) -> bool {
        match self.kind {
            IntegerExpression(_, _) | FloatExpression(_, _) => true,
            _ => false,
        }
    }

    pub fn declaration_kind(&self) -> DeclarationKind {
        match self.kind {
            Class { .. } => DeclarationKind::Any,
            TypeParameter { .. } | ReferenceTypeExpression { .. } => DeclarationKind::Type,
            ParameterPattern { .. } | ReferenceExpression { .. } | LetBinding { .. } => {
                DeclarationKind::Value
            }
            _ => DeclarationKind::None,
        }
    }

    pub fn is_declaration(&self, declaration_kind: DeclarationKind) -> bool {
        match self.kind {
            Class { .. } => true,
            TypeParameter { .. } => declaration_kind.is_type(),
            ParameterPattern { .. } | LetBinding { .. } => declaration_kind.is_value(),
            _ => false,
        }
    }

    pub fn is_reference(&self, kind: DeclarationKind) -> bool {
        match self.kind {
            ReferenceTypeExpression { .. } => kind.is_type(),
            ReferenceExpression { .. } => kind.is_value(),
            SelfTypeExpression(_) => kind.is_type(),
            SelfExpression(_) => kind.is_value(),
            _ => false,
        }
    }

    pub fn is_expression(&self) -> bool {
        match self.kind {
            ReferenceExpression { .. }
            | MessageSendExpression { .. }
            | SelfExpression(_)
            | StringExpression(_, _)
            | CharacterExpression(_, _)
            | IntegerExpression(_, _)
            | FloatExpression(_, _) => true,
            _ => false,
        }
    }

    pub fn is_type_expression(&self) -> bool {
        match self.kind {
            ReferenceTypeExpression { .. } => true,
            _ => false,
        }
    }

    pub fn is_message(&self) -> bool {
        match self.kind {
            UnaryMessage { .. } | BinaryMessage { .. } | KeywordMessage { .. } => true,
            _ => false,
        }
    }

    pub fn is_message_pattern(&self) -> bool {
        match self.kind {
            UnaryMessagePattern { .. }
            | BinaryMessagePattern { .. }
            | KeywordMessagePattern { .. } => true,
            _ => false,
        }
    }
}

impl fmt::Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} @ {}: {:?}", self.id, self.span, self.kind)
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum DeclarationKind {
    Type,
    Value,
    Any,
    None,
}

impl DeclarationKind {
    pub fn is_value(&self) -> bool {
        match self {
            DeclarationKind::Any | DeclarationKind::Value => true,
            DeclarationKind::Type | DeclarationKind::None => false,
        }
    }

    pub fn is_type(&self) -> bool {
        match self {
            DeclarationKind::Any | DeclarationKind::Type => true,
            DeclarationKind::Value | DeclarationKind::None => false,
        }
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
    /// REPLLine ::=
    ///   REPLStatement+
    /// ```
    REPLLine { statements: Vec<Id> },

    /// ```bnf
    /// REPLStatement ::=
    ///   REPLDirective |
    ///   LetBinding |
    ///   REPLExpression |
    ///   ImportDirective |
    ///   Declaration
    /// ```

    /// ```bnf
    /// REPLDirective ::=
    ///   COLON
    ///   Symbol
    ///   Expression
    ///   PERIOD?
    /// ```
    REPLDirective {
        colon: Token,
        symbol: Id,
        expression: Id,
        period: Option<Token>,
    },

    /// ```bnf
    /// REPLExpression ::=
    ///   Expression
    ///   PERIOD?
    /// ```
    REPLExpression {
        expression: Id,
        period: Option<Token>,
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
    ///   PARTIAL_KEYWORD?
    ///   CLASS_KEYWORD
    ///   Symbol
    ///   TypeParameterList?
    ///   (ClassBody | PERIOD)
    /// ```
    Class {
        partial_keyword: Option<Token>,
        class_keyword: Option<Token>,
        symbol: Id,
        type_parameter_list: Id,
        class_body: Id,
        period: Option<Token>,
    },

    /// ```bnf
    /// TypeParameterList ::=
    ///   OPEN_ANGLE
    ///   TypeParameter
    ///   (COMMA TypeParameter)*
    ///   CLOSE_ANGLE
    /// ```
    TypeParameterList {
        open_angle: Option<Token>,
        type_parameters: Vec<Id>,
        close_angle: Option<Token>,
    },

    /// ```bnf
    /// TypeParameter ::=
    ///   Symbol
    ///   (IN_KEYWORD | OUT_KEYWORD | INOUT_KEYWORD)?
    /// ```
    TypeParameter {
        symbol: Id,
        variance_keyword: Option<Token>,
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
    ///   Method |
    ///   IsDirective
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
    /// IsDirective ::=
    ///   IS_KEYWORD
    ///   TypeExpression
    ///   PERIOD
    /// ```
    IsDirective {
        is_keyword: Token,
        type_expression: Id,
        period: Option<Token>,
    },

    /// ```bnf
    /// Signature ::=
    ///   TypeParameterList?
    ///   MessagePattern
    ///   ReturnType?
    /// ```
    Signature {
        type_parameter_list: Id,
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
    Operator(Vec<Token>),

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
    ///   ReferenceTypeExpression |
    ///   SelfTypeExpression |
    ///   Nothing
    /// ```

    /// ```bnf
    /// ReferenceTypeExpression ::=
    ///   Symbol
    ///   TypeArgumentList?
    /// ```
    ReferenceTypeExpression { symbol: Id, type_argument_list: Id },

    /// ```bnf
    /// SelfTypeExpression ::=
    ///   SELF_KEYWORD
    /// ```
    SelfTypeExpression(Token),

    /// ```bnf
    /// Nothing ::=
    ///   UNDERSCORE
    /// ```
    Nothing(Token),

    /// ```bnf
    /// TypeArgumentList ::=
    ///   OPEN_ANGLE
    ///   TypeExpression
    ///   (COMMA TypeExpression)*
    ///   CLOSE_ANGLE
    /// ```
    TypeArgumentList {
        open_angle: Option<Token>,
        type_expressions: Vec<Id>,
        close_angle: Option<Token>,
    },

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
    ///   MessageSendExpression |
    ///   SelfExpression |
    ///   StringExpression |
    ///   CharacterExpression |
    ///   IntegerExpression |
    ///   FloatExpression
    /// ```

    /// ```bnf
    /// ReferenceExpression ::=
    ///   Symbol
    /// ```
    ReferenceExpression { symbol: Id },

    /// ```bnf
    /// SelfExpression ::=
    ///   SELF_KEYWORD
    /// ```
    SelfExpression(Token),

    /// ```bnf
    /// StringExpression ::=
    ///   SIMPLE_STRING
    /// ```
    StringExpression(Token, String),

    /// ```bnf
    /// CharacterExpression ::=
    ///   SIMPLE_CHARACTER
    /// ```
    CharacterExpression(Token, Option<u16>),

    /// ```bnf
    /// IntegerExpression ::=
    ///   SIMPLE_INTEGER
    /// ```
    IntegerExpression(Token, BigInt),

    /// ```bnf
    /// FloatExpression ::=
    ///   SIMPLE_FLOAT
    /// ```
    FloatExpression(Token, BigFraction),

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

    /// ```bnf
    /// LetExpression ::=
    ///   LetBinding
    ///   Expression
    /// ```
    LetExpression { let_binding: Id, expression: Id },

    /// ```bnf
    /// LetBinding ::=
    ///   LET_KEYWORD
    ///   TypeExpression?
    ///   Symbol
    ///   EQUAL_SIGN
    ///   Expression
    ///   Period
    /// ```
    LetBinding {
        let_keyword: Option<Token>,
        type_expression: Id,
        symbol: Id,
        equal_sign: Option<Token>,
        expression: Id,
        period: Option<Token>,
    },
}

pub use NodeKind::*;

impl NodeKind {
    pub fn leaves(&self) -> Vec<&Token> {
        let option_tokens: Vec<Option<&Token>> = match self {
            REPLDirective {
                ref colon,
                ref period,
                ..
            } => vec![Some(colon), period.as_ref()],

            REPLExpression { ref period, .. } => vec![period.as_ref()],

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
                ref partial_keyword,
                ref class_keyword,
                ref period,
                ..
            } => vec![
                partial_keyword.as_ref(),
                class_keyword.as_ref(),
                period.as_ref(),
            ],

            TypeParameterList {
                ref open_angle,
                ref close_angle,
                ..
            } => vec![open_angle.as_ref(), close_angle.as_ref()],

            TypeParameter {
                ref variance_keyword,
                ..
            } => vec![variance_keyword.as_ref()],

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

            IsDirective {
                ref is_keyword,
                ref period,
                ..
            } => vec![Some(is_keyword), period.as_ref()],

            Operator(ref tokens) => tokens.iter().map(Some).collect(),

            KeywordPair { ref colon, .. } => vec![colon.as_ref()],

            ReturnType { ref arrow, .. } => vec![arrow.as_ref()],

            MethodBody { ref fat_arrow, .. } => vec![fat_arrow.as_ref()],

            Nothing(ref underscore) => vec![Some(underscore)],

            SelfExpression(ref keyword) => vec![Some(keyword)],

            SelfTypeExpression(ref keyword) => vec![Some(keyword)],

            TypeArgumentList {
                ref open_angle,
                ref close_angle,
                ..
            } => vec![open_angle.as_ref(), close_angle.as_ref()],

            StringExpression(ref token, _) => vec![Some(token)],

            CharacterExpression(ref token, _) => vec![Some(token)],

            IntegerExpression(ref token, _) => vec![Some(token)],

            FloatExpression(ref token, _) => vec![Some(token)],

            LetExpression { .. } => vec![],

            LetBinding {
                ref let_keyword,
                ref equal_sign,
                ref period,
                ..
            } => vec![let_keyword.as_ref(), equal_sign.as_ref(), period.as_ref()],

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
            REPLLine { statements, .. } => {
                children.extend(statements);
            }
            REPLDirective {
                symbol, expression, ..
            } => {
                children.push(symbol);
                children.push(expression);
            }
            REPLExpression { expression, .. } => {
                children.push(expression);
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
                symbol,
                type_parameter_list,
                class_body,
                ..
            } => {
                children.push(symbol);
                children.push(type_parameter_list);
                children.push(class_body);
            }
            TypeParameterList {
                type_parameters, ..
            } => {
                children.extend(type_parameters);
            }
            TypeParameter { symbol, .. } => {
                children.push(symbol);
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
            IsDirective {
                type_expression, ..
            } => {
                children.push(type_expression);
            }
            Signature {
                type_parameter_list,
                message_pattern,
                return_type,
            } => {
                children.push(type_parameter_list);
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
            SelfExpression(_) => {}
            SelfTypeExpression(_) => {}
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
            ReferenceTypeExpression {
                symbol,
                type_argument_list,
            } => {
                children.push(symbol);
                children.push(type_argument_list);
            }
            Nothing(_) => {}
            TypeArgumentList {
                type_expressions, ..
            } => {
                children.extend(type_expressions);
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
            StringExpression(_, _) => {}
            CharacterExpression(_, _) => {}
            IntegerExpression(_, _) => {}
            FloatExpression(_, _) => {}
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
            LetExpression {
                let_binding,
                expression,
                ..
            } => {
                children.push(let_binding);
                children.push(expression);
            }
            LetBinding {
                type_expression,
                symbol,
                expression,
                ..
            } => {
                children.push(type_expression);
                children.push(symbol);
                children.push(expression);
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
        fix_parentage(&mut self.tree, id, self.parent_id, self.start);
        id
    }
}

fn fix_parentage(tree: &mut Tree, id: Id, parent_id: Option<Id>, start: Location) {
    if let Some(node) = tree.get_mut(id) {
        node.parent_id = parent_id;
        if let MessageSendExpression { .. } = node.kind {
            node.span.start = start;
        }
        let start = node.span.start.clone();
        for child in node.children() {
            fix_parentage(tree, child, Some(id), start.clone());
        }
    }
}
