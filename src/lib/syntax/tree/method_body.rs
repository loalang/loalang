use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct MethodBody {
    pub id: Id,
}

impl Node for MethodBody {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        unimplemented!()
    }

    fn children(&self) -> Vec<&dyn Node> {
        unimplemented!()
    }
}
