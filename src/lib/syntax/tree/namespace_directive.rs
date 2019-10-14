use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct NamespaceDirective {
    pub id: Id,
    pub namespace_keyword: Option<Token>,
    pub qualified_symbol: Option<QualifiedSymbol>,
    pub period: Option<Token>,
}

impl Node for NamespaceDirective {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

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
