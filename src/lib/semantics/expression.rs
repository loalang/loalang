use crate::semantics::*;
use crate::*;

pub enum Expression {
    Integer(BigInt),
    MessageSend(Arc<Expression>, Message),
    Reference(Reference),
}
