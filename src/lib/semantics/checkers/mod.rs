use crate::semantics::Checker;

mod undefined_type_reference;
pub use self::undefined_type_reference::*;

mod undefined_reference;
pub use self::undefined_reference::*;

const UNDEFINED_TYPE_REFERENCE: UndefinedTypeReference = UndefinedTypeReference;
const UNDEFINED_REFERENCE: UndefinedReference = UndefinedReference;

#[inline]
pub fn checkers() -> Vec<&'static dyn Checker> {
    vec![&UNDEFINED_TYPE_REFERENCE, &UNDEFINED_REFERENCE]
}
