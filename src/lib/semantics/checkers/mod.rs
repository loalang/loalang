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

mod invalid_import;
pub use self::invalid_import::*;

mod valid_inherit;
pub use self::valid_inherit::*;

mod out_of_bounds_number;
pub use self::out_of_bounds_number::*;

mod imprecise_float_literal;
pub use self::imprecise_float_literal::*;

const UNDEFINED_TYPE_REFERENCE: UndefinedTypeReference = UndefinedTypeReference;
const UNDEFINED_REFERENCE: UndefinedReference = UndefinedReference;
const UNDEFINED_BEHAVIOUR: UndefinedBehaviour = UndefinedBehaviour;
const TYPE_ASSIGNMENT: TypeAssignment = TypeAssignment;
const DUPLICATE_DECLARATION: DuplicateDeclaration = DuplicateDeclaration;
const INVALID_IMPORT: InvalidImport = InvalidImport;
const INVALID_INHERIT: InvalidInherit = InvalidInherit;
const OUT_OF_BOUNDS_NUMBER: OutOfBoundsNumber = OutOfBoundsNumber;
const IMPRECISE_FLOAT_LITERAL: ImpreciseFloatLiteral = ImpreciseFloatLiteral;

#[inline]
pub fn checkers() -> Vec<&'static dyn Checker> {
    vec![
        &UNDEFINED_TYPE_REFERENCE,
        &UNDEFINED_REFERENCE,
        &UNDEFINED_BEHAVIOUR,
        &TYPE_ASSIGNMENT,
        &DUPLICATE_DECLARATION,
        &INVALID_IMPORT,
        &INVALID_INHERIT,
        &OUT_OF_BOUNDS_NUMBER,
        &IMPRECISE_FLOAT_LITERAL,
    ]
}
