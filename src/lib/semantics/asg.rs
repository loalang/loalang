use crate::*;
use crate::semantics::*;

pub enum Expression {
    Integer(BigInt),
    MessageSend(Arc<Expression>, Arc<Message>),
}

pub struct Message {
    pub selector: Symbol,
    pub arguments: Vec<Arc<Expression>>,
}

#[derive(Clone)]
pub enum TypeConstructor {
    Class(Arc<Class>),
    TypeParameter(Arc<TypeParameter>),
}

impl TypeConstructor {
    pub fn name(&self) -> &Symbol {
        match self {
            TypeConstructor::Class(class) => &class.name,
            TypeConstructor::TypeParameter(param) => &param.name,
        }
    }

    pub fn type_parameters(&self) -> &Vec<Arc<TypeParameter>> {
        match self {
            TypeConstructor::Class(class) => &class.type_parameters,
            TypeConstructor::TypeParameter(param) => &param.type_parameters,
        }
    }
}

pub struct TypeParameter {
    pub constraint: Type,
    pub name: Symbol,
    pub type_parameters: Vec<Arc<TypeParameter>>,
}

#[derive(Clone)]
pub enum Pattern {
    Binding(Type, Symbol),
}

pub struct Variable {
    pub name: Symbol,
    pub typ: Type,
}
