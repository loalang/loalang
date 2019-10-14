use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct ImportDirective {
    pub id: Id,
    pub import_keyword: Option<Token>,
    pub qualified_symbol: Option<QualifiedSymbol>,
    pub as_keyword: Option<Token>,
    pub symbol: Option<Symbol>,
    pub period: Option<Token>,
}

impl Node for ImportDirective {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

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
        } else if let Some(ref n) = self.period {
            first_node = n
        } else {
            return None;
        }

        if let Some(ref n) = self.period {
            last_node = n
        } else if let Some(ref n) = self.symbol {
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
        push!(children, self.period);

        children
    }
}
