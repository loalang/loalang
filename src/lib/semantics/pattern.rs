use crate::semantics::*;

#[derive(Clone)]
pub enum Pattern {
    Binding(Type, Symbol),
}

impl Pattern {
    pub fn typ(&self) -> Type {
        match self {
            Pattern::Binding(t, _) => t.clone(),
        }
    }
}
