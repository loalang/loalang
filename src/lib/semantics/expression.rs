use crate::*;
use crate::semantics::*;

pub enum Expression {
    Integer(BigInt),
    MessageSend(Arc<Expression>, Message),
}

