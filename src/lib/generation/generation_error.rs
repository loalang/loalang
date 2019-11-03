use crate::syntax::*;
use std::option::NoneError;

#[derive(Debug)]
pub enum GenerationError {
    TraversalFailure,
    InvalidNode(Node, String),
}

impl From<NoneError> for GenerationError {
    fn from(_: NoneError) -> Self {
        GenerationError::TraversalFailure
    }
}
