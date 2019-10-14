use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct ReturnType {
    pub id: Id,
    pub arrow: Option<Token>,
    pub type_expression: Option<TypeExpression>,
}

impl Node for ReturnType {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref n) = self.arrow {
            first_node = n
        } else if let Some(ref n) = self.type_expression {
            first_node = n
        } else {
            return None;
        }

        if let Some(ref n) = self.type_expression {
            last_node = n
        } else if let Some(ref n) = self.arrow {
            last_node = n
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.arrow);
        push!(children, self.type_expression);

        children
    }
}
