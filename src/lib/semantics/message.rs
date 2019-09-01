use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub struct Message {
    pub selector: Symbol,
    pub arguments: Vec<Arc<Expression>>,
}
