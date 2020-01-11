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

mod wrong_number_of_type_arguments;
pub use self::wrong_number_of_type_arguments::*;

mod private_methods;
pub use self::private_methods::*;

mod type_parameter_variance;
pub use self::type_parameter_variance::*;

const UNDEFINED_TYPE_REFERENCE: UndefinedTypeReference = UndefinedTypeReference;
const UNDEFINED_REFERENCE: UndefinedReference = UndefinedReference;
const UNDEFINED_BEHAVIOUR: UndefinedBehaviour = UndefinedBehaviour;
const TYPE_ASSIGNMENT: TypeAssignment = TypeAssignment;
const DUPLICATE_DECLARATION: DuplicateDeclaration = DuplicateDeclaration;
const INVALID_IMPORT: InvalidImport = InvalidImport;
const INVALID_INHERIT: InvalidInherit = InvalidInherit;
const OUT_OF_BOUNDS_NUMBER: OutOfBoundsNumber = OutOfBoundsNumber;
const IMPRECISE_FLOAT_LITERAL: ImpreciseFloatLiteral = ImpreciseFloatLiteral;
const WRONG_NUMBER_OF_TYPE_ARGUMENTS: WrongNumberOfTypeArguments = WrongNumberOfTypeArguments;
const PRIVATE_METHODS: PrivateMethods = PrivateMethods;
const TYPE_PARAMETER_VARIANCE: TypeParameterVariance = TypeParameterVariance;

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
        &WRONG_NUMBER_OF_TYPE_ARGUMENTS,
        &PRIVATE_METHODS,
        &TYPE_PARAMETER_VARIANCE,
    ]
}
