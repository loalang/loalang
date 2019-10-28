use crate::*;
use std::fmt;

#[derive(Clone)]
pub enum Diagnostic {
    SyntaxError(Span, String),
    UndefinedTypeReference(Span, String),
    UndefinedReference(Span, String),
}

impl Diagnostic {
    pub fn span(&self) -> &Span {
        use Diagnostic::*;

        match self {
            SyntaxError(ref s, _) => s,
            UndefinedTypeReference(ref s, _) => s,
            UndefinedReference(ref s, _) => s,
        }
    }

    pub fn level(&self) -> DiagnosticLevel {
        use Diagnostic::*;

        match self {
            SyntaxError(_, _) => DiagnosticLevel::Error,
            UndefinedTypeReference(_, _) => DiagnosticLevel::Error,
            UndefinedReference(_, _) => DiagnosticLevel::Error,
        }
    }

    pub fn code(&self) -> usize {
        use Diagnostic::*;

        match self {
            SyntaxError(_, _) => 1,
            UndefinedTypeReference(_, _) => 2,
            UndefinedReference(_, _) => 3,
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
            UndefinedTypeReference(_, s) => write!(f, "`{}` is undefined.", s),
            UndefinedReference(_, s) => write!(f, "`{}` is undefined.", s),
        }
    }
}

pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}
