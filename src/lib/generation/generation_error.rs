use crate::syntax::*;
use crate::*;
use std::option::NoneError;

#[derive(Debug)]
pub enum GenerationError {
    TraversalFailure,
    InvalidNode(Node, String),
    OutOfScope(Id),
}

impl From<NoneError> for GenerationError {
    fn from(_: NoneError) -> Self {
        #[cfg(debug_assertions)]
        panic!("Generation stopped because of faulty syntax tree.");
        #[cfg(not(debug_assertions))]
        GenerationError::TraversalFailure
    }
}
