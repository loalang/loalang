use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Method {
    pub id: Id,
    pub visibility: Option<Token>,
    pub signature: Signature,
    pub body: Option<MethodBody>,
    pub period: Option<Token>,
}

impl Node for Method {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref n) = self.visibility {
            first_node = n
        } else {
            first_node = &self.signature;
        }

        if let Some(ref n) = self.body {
            last_node = n
        } else {
            last_node = &self.signature;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.visibility);
        children.push(&self.signature);
        push!(children, self.body);

        children
    }
}
