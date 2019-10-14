use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum Declaration {
    Class(Class),
}

impl Node for Declaration {
    fn id(&self) -> Option<Id> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            Declaration::Class(ref c) => c.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Declaration::Class(ref c) => vec![c],
        }
    }
}
