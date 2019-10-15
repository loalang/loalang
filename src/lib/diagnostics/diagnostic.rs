use crate::*;
use std::fmt;

#[derive(Clone)]
pub enum Diagnostic {
    SyntaxError(Span, String),
}

impl Diagnostic {
    pub fn span(&self) -> &Span {
        use Diagnostic::*;

        match self {
            SyntaxError(ref s, _) => s,
        }
    }
}

impl ToString for Diagnostic {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Diagnostic::*;

        match self {
            SyntaxError(_, s) => write!(f, "{}", s),
        }
    }
}
