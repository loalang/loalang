use crate::semantics::*;

#[derive(Clone)]
pub enum Reference {
    Unresolved(Symbol),
    Binding(*const Binding),
    Class(*const Class),
}

impl Reference {
    pub fn symbol(&self) -> &Symbol {
        match self {
            Reference::Unresolved(ref s) => s,
            Reference::Binding(b) => {
                let Binding(_, ref s) = unsafe { &**b };
                s
            }
            Reference::Class(c) => &unsafe { &**c }.name,
        }
    }
}
