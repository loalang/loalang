use crate::*;

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum TokenKind {
    EOF,
    Whitespace(String),
}
