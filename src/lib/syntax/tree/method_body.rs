use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct MethodBody {
    pub id: Id,
    pub fat_arrow: Option<Token>,
    pub expression: Option<Expression>,
}

impl Node for MethodBody {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref n) = self.fat_arrow {
            first_node = n;
        } else if let Some(ref n) = self.expression {
            first_node = n;
        } else {
            return None;
        }

        if let Some(ref n) = self.expression {
            last_node = n;
        } else if let Some(ref n) = self.fat_arrow {
            last_node = n;
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.fat_arrow);
        push!(children, self.expression);

        children
    }
}
