use crate::semantics::Checker;

mod undefined_direct_reference;
pub use self::undefined_direct_reference::*;

const UNDEFINED_DIRECT_REFERENCE: UndefinedDirectReference = UndefinedDirectReference;

#[inline]
pub fn checkers() -> Vec<&'static dyn Checker> {
    vec![&UNDEFINED_DIRECT_REFERENCE]
}
