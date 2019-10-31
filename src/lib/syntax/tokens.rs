use crate::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    EOF,
    Unknown(u16),
    Whitespace(String),
    LineComment(String),

    AsKeyword,
    InKeyword,
    IsKeyword,
    OutKeyword,
    InoutKeyword,
    ClassKeyword,
    PrivateKeyword,
    PublicKeyword,
    NamespaceKeyword,
    SelfKeyword,
    ImportKeyword,
    ExportKeyword,
    PartialKeyword,

    Plus,
    Colon,
    Comma,
    Period,
    Slash,
    EqualSign,

    Arrow,
    FatArrow,

    OpenAngle,
    CloseAngle,
    OpenCurly,
    CloseCurly,

    SimpleInteger(String),
    SimpleSymbol(String),

    Underscore,
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

            AsKeyword => "as".into(),
            InKeyword => "in".into(),
            IsKeyword => "is".into(),
            OutKeyword => "out".into(),
            InoutKeyword => "inout".into(),
            ClassKeyword => "class".into(),
            PrivateKeyword => "private".into(),
            PublicKeyword => "public".into(),
            NamespaceKeyword => "namespace".into(),
            SelfKeyword => "self".into(),
            ImportKeyword => "import".into(),
            ExportKeyword => "export".into(),
            PartialKeyword => "partial".into(),

            Plus => "+".into(),
            Colon => ":".into(),
            Comma => ",".into(),
            Period => ".".into(),
            Slash => "/".into(),
            EqualSign => "=".into(),

            Arrow => "->".into(),
            FatArrow => "=>".into(),

            OpenAngle => "<".into(),
            CloseAngle => ">".into(),
            OpenCurly => "{".into(),
            CloseCurly => "}".into(),

            Underscore => "_".into(),

            LineComment(s) => format!("//{}", s),

            Whitespace(s) | SimpleInteger(s) | SimpleSymbol(s) => s.clone(),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}
