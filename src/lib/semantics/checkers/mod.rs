use crate::semantics::Checker;

mod undefined_type_reference;
pub use self::undefined_type_reference::*;

const UNDEFINED_TYPE_REFERENCE: UndefinedTypeReference = UndefinedTypeReference;

#[inline]
pub fn checkers() -> Vec<&'static dyn Checker> {
    vec![&UNDEFINED_TYPE_REFERENCE]
}
