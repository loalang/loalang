use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum ClassMember {
    Method(Method),
}

impl Node for ClassMember {
    fn id(&self) -> Option<Id> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            ClassMember::Method(ref m) => m.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ClassMember::Method(ref m) => vec![m],
        }
    }
}
