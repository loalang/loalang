use crate::semantics::Checker;

mod undefined_type_reference;
pub use self::undefined_type_reference::*;

mod undefined_reference;
pub use self::undefined_reference::*;

mod type_assignment;
pub use self::type_assignment::*;

const UNDEFINED_TYPE_REFERENCE: UndefinedTypeReference = UndefinedTypeReference;
const UNDEFINED_REFERENCE: UndefinedReference = UndefinedReference;
const TYPE_ASSIGNMENT: TypeAssignment = TypeAssignment;

#[inline]
pub fn checkers() -> Vec<&'static dyn Checker> {
    vec![
        &UNDEFINED_TYPE_REFERENCE,
        &UNDEFINED_REFERENCE,
        &TYPE_ASSIGNMENT,
    ]
}
