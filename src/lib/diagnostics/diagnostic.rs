use crate::*;
use std::fmt;

pub enum Diagnostic {
    UnexpectedToken(syntax::Token, String),
    UndefinedSymbol(semantics::Symbol),
    MissingBehaviour(semantics::Type, semantics::Symbol),
}

impl Diagnostic {
    pub fn span(&self) -> Option<Span> {
        use Diagnostic::*;

        match self {
            UnexpectedToken(t, _) => Some(t.span.clone()),
            UndefinedSymbol(semantics::Symbol(s, _)) => s.clone(),
            MissingBehaviour(_, semantics::Symbol(s, _)) => s.clone(),
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
            UnexpectedToken(token, expected) => {
                write!(f, "Unexpected {:?}; expected {}.", token, expected)
            }

            UndefinedSymbol(symbol) => write!(f, "`{}` is undefined.", symbol),

            MissingBehaviour(typ, symbol) => {
                write!(f, "`{}` doesn't respond to `{}`.", typ, symbol)
            }
        }
    }
}
