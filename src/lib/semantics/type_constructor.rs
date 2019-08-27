use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub enum TypeConstructor {
    Unresolved(Symbol),

    Class(Arc<Class>),
    TypeParameter(Arc<TypeParameter>),
}

impl TypeConstructor {
    pub fn name(&self) -> &Symbol {
        match self {
            TypeConstructor::Unresolved(s) => &s,

            TypeConstructor::Class(class) => &class.name,
            TypeConstructor::TypeParameter(param) => &param.name,
        }
    }

    pub fn type_parameters(&self) -> Cow<Vec<Arc<TypeParameter>>> {
        match self {
            TypeConstructor::Unresolved(_) => Cow::Owned(vec![]),

            TypeConstructor::Class(class) => Cow::Borrowed(&class.type_parameters),
            TypeConstructor::TypeParameter(param) => Cow::Borrowed(&param.type_parameters),
        }
    }
}
