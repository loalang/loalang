use crate::syntax::*;
use std::fmt;

pub enum Diagnostic {
    UnexpectedToken(Token, String),
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Diagnostic::*;

        match self {
            UnexpectedToken(token, expected) => write!(f, "Unexpected {:?}; expected {}", token, expected)
        }
    }
}
