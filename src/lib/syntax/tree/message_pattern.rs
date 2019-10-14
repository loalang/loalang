use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum MessagePattern {
    Unary(Id, Symbol),
}

impl Node for MessagePattern {
    fn id(&self) -> Option<Id> {
        match self {
            MessagePattern::Unary(ref id, _) => Some(*id),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            MessagePattern::Unary(_, s) => s.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            MessagePattern::Unary(_, s) => vec![s],
        }
    }
}
