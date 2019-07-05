use crate::*;
use crate::semantics::*;

pub struct Message {
    pub selector: Symbol,
    pub arguments: Vec<Arc<Expression>>,
}

