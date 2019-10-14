use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct QualifiedSymbol {
    pub id: Id,
    pub symbols: Vec<Symbol>,
}

impl Node for QualifiedSymbol {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

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
