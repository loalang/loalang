use crate::semantics::Checker;

mod undefined_type_reference;
pub use self::undefined_type_reference::*;

mod undefined_reference;
pub use self::undefined_reference::*;

mod undefined_behaviour;
pub use self::undefined_behaviour::*;

mod type_assignment;
pub use self::type_assignment::*;

mod duplicate_declaration;
pub use self::duplicate_declaration::*;

const UNDEFINED_TYPE_REFERENCE: UndefinedTypeReference = UndefinedTypeReference;
const UNDEFINED_REFERENCE: UndefinedReference = UndefinedReference;
const UNDEFINED_BEHAVIOUR: UndefinedBehaviour = UndefinedBehaviour;
const TYPE_ASSIGNMENT: TypeAssignment = TypeAssignment;
const DUPLICATE_DECLARATION: DuplicateDeclaration = DuplicateDeclaration;

#[inline]
pub fn checkers() -> Vec<&'static dyn Checker> {
    vec![
        &UNDEFINED_TYPE_REFERENCE,
        &UNDEFINED_REFERENCE,
        &UNDEFINED_BEHAVIOUR,
        &TYPE_ASSIGNMENT,
        &DUPLICATE_DECLARATION,
    ]
}
