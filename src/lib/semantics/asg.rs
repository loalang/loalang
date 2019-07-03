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
}

pub struct TypeParameter {
    pub constraint: Type,
    pub name: Symbol,
    pub parameters: Vec<Arc<TypeParameter>>,
}

pub struct Signature {
    pub selector: Symbol,
    pub type_parameters: Vec<Arc<TypeParameter>>,
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.selector, self.return_type)
    }
}

pub enum MethodImplementation {
    Body(Vec<Pattern>, Arc<Expression>),
    VariableGetter(Arc<Variable>),
    VariableSetter(Arc<Variable>),
}

pub enum Pattern {
    Binding(Type, Symbol),
}

pub struct Variable {
    pub name: Symbol,
    pub typ: Type,
}
