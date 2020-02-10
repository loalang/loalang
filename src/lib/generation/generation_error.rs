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
    fn from(e: NoneError) -> Self {
        panic!(e)
        // GenerationError::TraversalFailure
    }
}
