use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum Expression {
    Reference(Id, Symbol),
}

impl Node for Expression {
    fn id(&self) -> Option<Id> {
        match self {
            Expression::Reference(id, _) => Some(*id),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            Expression::Reference(_, s) => s.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Expression::Reference(_, s) => vec![s],
        }
    }
}
