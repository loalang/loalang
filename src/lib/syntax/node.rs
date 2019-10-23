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

    pub fn child_nodes(&self, tree: Arc<Tree>) -> Vec<Node> {
        let mut out = vec![];
        for child_id in self.children() {
            if let Some(n) = tree.get(child_id) {
                out.push(n);
            }
        }
        out
    }

    pub fn symbol_id(&self) -> Option<Id> {
        match self.kind {
            Class { symbol, .. } | ReferenceTypeExpression { symbol, .. } => Some(symbol),
            _ => None,
        }
    }

    /// Traverses all nodes in the tree below this point.
    /// If the callback returns true for a given node, the
    /// traversal will continue down its children. Otherwise,
    /// the traversal will not traverse down that path.
    pub fn traverse<F: FnMut(&Node) -> bool>(&self, tree: Arc<Tree>, f: &mut F) {
        if !f(self) {
            return;
        }

        for child in self.child_nodes(tree.clone()) {
            child.traverse(tree.clone(), f);
        }
    }

    pub fn closest_upwards<F: Fn(&Node) -> bool>(&self, tree: Arc<Tree>, f: F) -> Option<Node> {
        if f(self) {
            return Some(self.clone());
        }
        let mut parent = self.parent_id?;
        loop {
            let parent_node = tree.get(parent)?;
            if f(&parent_node) {
                return Some(parent_node.clone());
            }
            for child in parent_node.child_nodes(tree.clone()) {
                if f(&child) {
                    return Some(child.clone());
                }
            }
            parent = parent_node.parent_id?;
        }
    }

    pub fn all_downwards<F: Fn(&Node) -> bool>(&self, tree: Arc<Tree>, f: &F) -> Vec<Node> {
        let mut nodes = vec![];

        if f(self) {
            nodes.push(self.clone());
        }

        for child in self.child_nodes(tree.clone()) {
            nodes.extend(child.all_downwards(tree.clone(), f));
        }

        nodes
    }

    pub fn is_scope_root(&self) -> bool {
        match self.kind {
            Module { .. } | ClassBody { .. } | Method { .. } => true,
            _ => false,
        }
    }

    pub fn closest_scope_root_upwards(&self, tree: Arc<Tree>) -> Option<Node> {
        self.closest_upwards(tree, |n| n.is_scope_root())
    }

    pub fn all_scope_roots_downwards(&self, tree: Arc<Tree>) -> Vec<Node> {
        self.all_downwards(tree, &|n| n.is_scope_root())
    }

    pub fn is_declaration(&self) -> bool {
        match self.kind {
            Class { .. } | ParameterPattern { .. } => true,
            _ => false,
        }
    }

    pub fn is_import_directive(&self) -> bool {
        match self.kind {
            ImportDirective { .. } => true,
            _ => false,
        }
    }

    pub fn closest_declaration_upwards(&self, tree: Arc<Tree>) -> Option<Node> {
        self.closest_upwards(tree, |n| n.is_declaration())
    }

    pub fn all_declarations_downwards(&self, tree: Arc<Tree>) -> Vec<Node> {
        self.all_downwards(tree, &|n| n.is_declaration())
    }

    pub fn is_reference(&self) -> bool {
        match self.kind {
            ReferenceTypeExpression { .. } => true,
            _ => false,
        }
    }

    pub fn closest_references_upwards(&self, tree: Arc<Tree>) -> Option<Node> {
        self.closest_upwards(tree, |n| n.is_reference())
    }

    pub fn all_references_downwards(&self, tree: Arc<Tree>) -> Vec<Node> {
        self.all_downwards(tree, &|n| n.is_reference())
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
}

pub use NodeKind::*;

impl NodeKind {
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
}
