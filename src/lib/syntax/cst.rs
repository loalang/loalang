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

#[derive(Debug)]
pub enum Method {
    Concrete(ConcreteMethod),
}

#[derive(Debug)]
pub struct ConcreteMethod(
    pub Option<TypeParameterList>,
    pub MessagePattern,
    pub Option<ReturnType>,
    pub MethodBody,
);

#[derive(Debug)]
pub struct TypeParameterList(pub Token, pub Vec<TypeParameter>, pub Token);

#[derive(Debug)]
pub struct TypeArgumentList(pub Token, pub Vec<Type>, pub Token);

#[derive(Debug)]
pub struct TypeParameter(pub Option<Type>, pub Identifier, pub Option<Variance>);

#[derive(Debug)]
pub enum Variance {
    Inout(Token),
    In(Token),
    Out(Token),
}

#[derive(Debug)]
pub enum Type {
    Class(Identifier, Option<TypeArgumentList>),
}

#[derive(Debug)]
pub enum MessagePattern {
    Unary(Identifier),
    Binary(Token, Pattern),
    Keyword(Keyworded<Pattern>),
}

#[derive(Debug)]
pub struct ReturnType(pub Token, pub Type);

#[derive(Debug)]
pub struct MethodBody(pub Token, pub Expression);

#[derive(Debug)]
pub enum Pattern {
    Binding(Option<Type>, Identifier),
}

#[derive(Debug)]
pub struct Class(pub Token, pub Identifier, pub Option<TypeParameterList>, pub ClassBody);

#[derive(Debug)]
pub enum ClassBody {
    Empty(Token),
    Braced(Token, Vec<ClassMember>, Token)
}

#[derive(Debug)]
pub enum ClassMember {
    Method(Token, Method, Token),
}
