use crate::syntax::*;
use crate::*;

macro_rules! push_all {
    ($children:expr, $other:expr) => {
        $children.extend($other.iter().map(|t| t as &dyn Node));
    };
}

macro_rules! push {
    ($children:expr, $option:expr) => {
        if let Some(ref o) = $option {
            $children.push(o);
        }
    };
}

pub trait Node {
    fn span(&self) -> Option<Span>;
    fn children(&self) -> Vec<&dyn Node>;
    fn contains_location(&self, location: &Location) -> bool {
        match self.span() {
            None => false,
            Some(s) => s.contains_location(location),
        }
    }
    fn is_token(&self) -> bool {
        false
    }
}

pub fn traverse<'b>(node: &'b dyn Node) -> NodeTraverser<'b> {
    NodeTraverser {
        nodes: vec![node],
        current: None,
    }
}

pub struct NodeTraverser<'a> {
    nodes: Vec<&'a dyn Node>,
    current: Option<Box<NodeTraverser<'a>>>,
}

impl<'a> Iterator for NodeTraverser<'a> {
    type Item = &'a dyn Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => {
                if self.nodes.len() == 0 {
                    return None;
                }
                let n = self.nodes.remove(0);
                self.current = Some(Box::new(NodeTraverser {
                    nodes: n.children(),
                    current: None,
                }));

                return Some(n);
            }

            Some(ref mut current) => match current.next() {
                None => {
                    self.current = None;
                    self.next()
                }

                Some(i) => Some(i),
            },
        }
    }
}

impl Node for Token {
    fn span(&self) -> Option<Span> {
        Some(self.span.clone())
    }

    fn children(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn is_token(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct Module {
    pub id: Id,
    pub namespace_directive: Option<NamespaceDirective>,
    pub import_directives: Vec<ImportDirective>,
    pub module_declarations: Vec<ModuleDeclaration>,
}

impl Node for Module {
    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref nd) = self.namespace_directive {
            first_node = nd;
        } else if let Some(id) = self.import_directives.first() {
            first_node = id;
        } else if let Some(md) = self.module_declarations.first() {
            first_node = md;
        } else {
            return None;
        }

        if let Some(md) = self.module_declarations.last() {
            last_node = md;
        } else if let Some(id) = self.import_directives.last() {
            last_node = id;
        } else if let Some(ref nd) = self.namespace_directive {
            last_node = nd;
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.namespace_directive);
        push_all!(children, self.import_directives);
        push_all!(children, self.module_declarations);

        children
    }
}

#[derive(Debug)]
pub struct NamespaceDirective {
    pub id: Id,
    pub namespace_keyword: Option<Token>,
    pub qualified_symbol: Option<QualifiedSymbol>,
    pub period: Option<Token>,
}

impl Node for NamespaceDirective {
    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref nk) = self.namespace_keyword {
            first_node = nk;
        } else if let Some(ref qs) = self.qualified_symbol {
            first_node = qs;
        } else if let Some(ref t) = self.period {
            first_node = t;
        } else {
            return None;
        }

        if let Some(ref p) = self.period {
            last_node = p;
        } else if let Some(ref qs) = self.qualified_symbol {
            last_node = qs;
        } else if let Some(ref nk) = self.namespace_keyword {
            last_node = nk;
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.namespace_keyword);
        push!(children, self.qualified_symbol);
        push!(children, self.period);

        children
    }
}

#[derive(Debug)]
pub struct ImportDirective {
    pub id: Id,
    pub import_keyword: Option<Token>,
    pub qualified_symbol: Option<QualifiedSymbol>,
    pub as_keyword: Option<Token>,
    pub symbol: Option<Symbol>,
}

impl Node for ImportDirective {
    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref n) = self.import_keyword {
            first_node = n
        } else if let Some(ref n) = self.qualified_symbol {
            first_node = n
        } else if let Some(ref n) = self.as_keyword {
            first_node = n
        } else if let Some(ref n) = self.symbol {
            first_node = n
        } else {
            return None;
        }

        if let Some(ref n) = self.symbol {
            last_node = n
        } else if let Some(ref n) = self.as_keyword {
            last_node = n
        } else if let Some(ref n) = self.qualified_symbol {
            last_node = n
        } else if let Some(ref n) = self.import_keyword {
            last_node = n
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.import_keyword);
        push!(children, self.qualified_symbol);
        push!(children, self.as_keyword);
        push!(children, self.symbol);

        children
    }
}

#[derive(Debug)]
pub struct QualifiedSymbol {
    pub id: Id,
    pub symbols: Vec<Symbol>,
}

impl Node for QualifiedSymbol {
    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(n) = self.symbols.first() {
            first_node = n
        } else {
            return None;
        }

        if let Some(n) = self.symbols.last() {
            last_node = n
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push_all!(children, self.symbols);

        children
    }
}

#[derive(Debug)]
pub struct Symbol {
    pub id: Id,
    pub token: Token,
}

impl Node for Symbol {
    fn span(&self) -> Option<Span> {
        self.token.span()
    }

    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.token]
    }
}

#[derive(Debug)]
pub enum ModuleDeclaration {
    Exported(Token, Declaration),
    NotExported(Declaration),
}

impl Node for ModuleDeclaration {
    fn span(&self) -> Option<Span> {
        unimplemented!()
    }

    fn children(&self) -> Vec<&dyn Node> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub enum Declaration {
    Class(ClassDeclaration),
}

impl Node for Declaration {
    fn span(&self) -> Option<Span> {
        unimplemented!()
    }

    fn children(&self) -> Vec<&dyn Node> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct ClassDeclaration {
    pub id: Id,
    pub class_keyword: Option<Token>,
    pub symbol: Option<Symbol>,
    pub period: Option<Token>,
}

impl Node for ClassDeclaration {
    fn span(&self) -> Option<Span> {
        unimplemented!()
    }

    fn children(&self) -> Vec<&dyn Node> {
        unimplemented!()
    }
}
