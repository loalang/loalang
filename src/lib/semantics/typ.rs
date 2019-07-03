use crate::*;
use crate::semantics::*;

#[derive(Clone)]
pub struct Type {
    pub constructor: TypeConstructor,
    pub arguments: Vec<Type>,
}

impl Type {
    pub fn callable_methods(&self) -> HashMap<Symbol, &Method> {
        match self.constructor {
            TypeConstructor::Class(ref class) => class.callable_methods(),
            TypeConstructor::TypeParameter(_) => HashMap::new(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.constructor.name())
    }
}
