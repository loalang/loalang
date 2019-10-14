use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Signature {
    pub id: Id,
    pub message_pattern: Option<MessagePattern>,
    pub return_type: Option<ReturnType>,
}

impl Node for Signature {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref n) = self.message_pattern {
            first_node = n
        } else if let Some(ref n) = self.return_type {
            first_node = n
        } else {
            return None;
        }

        if let Some(ref n) = self.return_type {
            last_node = n
        } else if let Some(ref n) = self.message_pattern {
            last_node = n
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.message_pattern);
        push!(children, self.return_type);

        children
    }
}
