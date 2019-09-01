use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub enum Pattern {
    Binding(Arc<Binding>),
}

impl Pattern {
    pub fn typ(&self) -> Type {
        match self {
            Pattern::Binding(b) => {
                let Binding(t, _) = &**b;
                t.clone()
            }
        }
    }
}

#[derive(Clone)]
pub struct Binding(pub Type, pub Symbol);
