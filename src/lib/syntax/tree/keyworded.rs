use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Keyworded<T> {
    pub id: Id,
    pub keywords: Vec<(Symbol, Token, T)>,
}

impl<T> Node for Keyworded<T>
where
    T: 'static + Node,
{
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some((n, _, _)) = self.keywords.first() {
            first_node = n;
        } else {
            return None;
        }

        if let Some((_, _, n)) = self.keywords.last() {
            last_node = n;
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        for (symbol, token, t) in self.keywords.iter() {
            children.push(symbol);
            children.push(token);
            children.push(t);
        }

        children
    }
}
