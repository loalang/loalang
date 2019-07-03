use crate::*;
use std::fmt;

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Debug, Clone)]
pub enum TokenKind {
    EOF,
    Unknown(char),
    Whitespace(String),
    LineComment(String),

    Plus,
    Colon,

    SimpleInteger(String),
    SimpleSymbol(String),
}
