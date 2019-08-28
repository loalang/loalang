use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub enum TypeConstructor {
    Unresolved(Symbol),

    Class(*const Class),
    TypeParameter(*const TypeParameter),
}

impl TypeConstructor {
    pub fn name(&self) -> &Symbol {
        match self {
            TypeConstructor::Unresolved(s) => &s,

            TypeConstructor::Class(class) => unsafe { &(**class).name },
            TypeConstructor::TypeParameter(param) => unsafe { &(**param).name },
        }
    }

    pub fn type_parameters(&self) -> Cow<Vec<Arc<TypeParameter>>> {
        match self {
            TypeConstructor::Unresolved(_) => Cow::Owned(vec![]),

            TypeConstructor::Class(class) => Cow::Borrowed(unsafe { &(**class).type_parameters }),
            TypeConstructor::TypeParameter(param) => {
                Cow::Borrowed(unsafe { &(**param).type_parameters })
            }
        }
    }
}
