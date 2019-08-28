use crate::semantics::*;
use crate::*;

pub struct TypeParameter {
    pub constraint: Type,
    pub name: Symbol,
    pub type_parameters: Vec<Arc<TypeParameter>>,
    pub variance: Variance,
}

pub enum Variance {
    Invariant,
    Covariant,
    Contravariant,
}
