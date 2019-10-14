use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum TypeExpression {
    Reference(Id, Symbol),
}

impl Node for TypeExpression {
    fn id(&self) -> Option<Id> {
        match self {
            TypeExpression::Reference(ref id, _) => Some(*id),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            TypeExpression::Reference(_, s) => s.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            TypeExpression::Reference(_, s) => vec![s],
        }
    }
}
