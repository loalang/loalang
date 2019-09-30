use crate::*;
use std::fmt;

#[derive(Debug, Clone)]
pub enum TokenKind {
    EOF,
    Unknown(char),
    Whitespace(String),
    LineComment(String),

    InKeyword,
    OutKeyword,
    InoutKeyword,
    ClassKeyword,
    PrivateKeyword,
    PublicKeyword,
    NamespaceKeyword,
    SelfKeyword,

    Plus,
    Colon,
    Comma,
    Period,
    Slash,

    Arrow,
    FatArrow,

    OpenAngle,
    CloseAngle,
    OpenCurly,
    CloseCurly,

    SimpleInteger(String),
    SimpleSymbol(String),
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn lexeme(&self) -> String {
        use TokenKind::*;

        match &self.kind {
            EOF => "\0".into(),
            Unknown(c) => c.to_string(),

            InKeyword => "in".into(),
            OutKeyword => "out".into(),
            InoutKeyword => "inout".into(),
            ClassKeyword => "class".into(),
            PrivateKeyword => "private".into(),
            PublicKeyword => "public".into(),
            NamespaceKeyword => "namespace".into(),
            SelfKeyword => "self".into(),

            Plus => "+".into(),
            Colon => ":".into(),
            Comma => ",".into(),
            Period => ".".into(),
            Slash => "/".into(),

            Arrow => "->".into(),
            FatArrow => "=>".into(),

            OpenAngle => "<".into(),
            CloseAngle => ">".into(),
            OpenCurly => "{".into(),
            CloseCurly => "}".into(),

            Whitespace(s) | LineComment(s) | SimpleInteger(s) | SimpleSymbol(s) => s.clone(),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}
