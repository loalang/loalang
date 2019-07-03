use crate::syntax::*;

#[derive(Debug)]
pub struct Integer(pub Token);

#[derive(Debug)]
pub struct Identifier(pub Token);

pub type Keyworded<T> = Box<[(Keyword, T)]>;

#[derive(Debug)]
pub enum MessageSend {
    Unary(Expression, Identifier),
    Binary(Expression, Token, Expression),
    Keyword(Expression, Keyworded<Expression>),
}

#[derive(Debug)]
pub struct Keyword(pub Identifier, pub Token);

#[derive(Debug)]
pub enum Expression {
    Integer(Integer),
    MessageSend(Box<MessageSend>),
}
