use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct ClassBody {
    pub id: Id,
    pub open_curly: Option<Token>,
    pub class_members: Vec<ClassMember>,
    pub close_curly: Option<Token>,
}

impl Node for ClassBody {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref n) = self.open_curly {
            first_node = n
        } else if let Some(n) = self.class_members.first() {
            first_node = n
        } else if let Some(ref n) = self.close_curly {
            first_node = n
        } else {
            return None;
        }

        if let Some(ref n) = self.close_curly {
            last_node = n
        } else if let Some(n) = self.class_members.last() {
            last_node = n
        } else if let Some(ref n) = self.open_curly {
            last_node = n
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.open_curly);
        push_all!(children, self.class_members);
        push!(children, self.close_curly);

        children
    }
}
