use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Module(pub NamespaceDirective, pub Vec<Class>);

#[derive(Debug)]
pub struct NamespaceDirective(pub Token, pub QualifiedIdentifier, pub Token);

#[derive(Debug)]
pub struct QualifiedIdentifier(pub Vec<Identifier>);

#[derive(Debug)]
pub struct Integer(pub Token);

impl Integer {
    pub fn span(&self) -> Span {
        let Integer(t) = self;
        t.span.clone()
    }
}

#[derive(Debug)]
pub struct Identifier(pub Token);

impl Identifier {
    pub fn span(&self) -> Span {
        let Identifier(t) = self;
        t.span.clone()
    }
}

pub type Keyworded<T> = Box<[(Keyword, T)]>;

#[derive(Debug)]
pub enum MessageSend {
    Unary(Expression, Identifier),
    Binary(Expression, Token, Expression),
    Keyword(Expression, Keyworded<Expression>),
}

impl MessageSend {
    pub fn span(&self) -> Span {
        use MessageSend::*;

        match self {
            Unary(e, i) => e.span().through(&i.span()),
            Binary(e, _, a) => e.span().through(&a.span()),
            Keyword(e, k) => e.span().through(&k[k.len() - 1].1.span()),
        }
    }
}

#[derive(Debug)]
pub struct Keyword(pub Identifier, pub Token);

#[derive(Debug)]
pub enum Expression {
    Integer(Integer),
    MessageSend(Box<MessageSend>),
    Reference(Identifier),
    SelfExpression(Token),
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::Integer(i) => i.span(),
            Expression::MessageSend(s) => s.span(),
            Expression::Reference(i) => i.span(),
            Expression::SelfExpression(t) => t.span.clone(),
        }
    }
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

impl TypeParameterList {
    pub fn span(&self) -> Span {
        let TypeParameterList(s, _, e) = self;
        s.span.through(&e.span)
    }
}

#[derive(Debug)]
pub struct TypeArgumentList(pub Token, pub Vec<Type>, pub Token);

impl TypeArgumentList {
    pub fn span(&self) -> Span {
        let TypeArgumentList(s, _, e) = self;
        s.span.through(&e.span)
    }
}

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

impl Type {
    pub fn span(&self) -> Span {
        use Type::*;

        match self {
            Class(i, Some(t)) => i.span().through(&t.span()),
            Class(i, None) => i.span(),
        }
    }
}

#[derive(Debug)]
pub enum MessagePattern {
    Unary(Identifier),
    Binary(Token, Pattern),
    Keyword(Keyworded<Pattern>),
}

impl MessagePattern {
    pub fn span(&self) -> Span {
        use MessagePattern::*;

        match self {
            Unary(i) => i.span(),
            Binary(t, p) => t.span.through(&p.span()),
            Keyword(k) => k[k.len() - 1].1.span(),
        }
    }
}

#[derive(Debug)]
pub struct ReturnType(pub Token, pub Type);

#[derive(Debug)]
pub struct MethodBody(pub Token, pub Expression);

#[derive(Debug)]
pub enum Pattern {
    Binding(Option<Type>, Identifier),
}

impl Pattern {
    pub fn span(&self) -> Span {
        use Pattern::*;

        match self {
            Binding(Some(t), i) => t.span().through(&i.span()),
            Binding(None, i) => i.span(),
        }
    }
}

#[derive(Debug)]
pub struct Class(
    pub Token,
    pub Identifier,
    pub Option<TypeParameterList>,
    pub ClassBody,
);

#[derive(Debug)]
pub enum ClassBody {
    Empty(Token),
    Braced(Token, Vec<ClassMember>, Token),
}

#[derive(Debug)]
pub enum ClassMember {
    Method(Token, Method, Token),
}
