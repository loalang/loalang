use crate::syntax::characters_to_string;
use crate::*;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    EOF,
    Unknown(u16),
    Whitespace(String),
    LineComment(String),
    DocComment(String),

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
    LetKeyword,
    NativeKeyword,

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
    SimpleFloat(String),
    SimpleString(String),
    SimpleCharacter(String),
    SimpleSymbol(String),
    SymbolLiteral(String),

    Underscore,
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub before: Vec<Token>,
    pub after: Vec<Token>,
}

impl Token {
    pub fn lexeme(&self) -> String {
        use TokenKind::*;

        match &self.kind {
            EOF => "\0".into(),
            Unknown(c) => characters_to_string([*c].iter().cloned()),

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
            LetKeyword => "let".into(),
            NativeKeyword => "native".into(),

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
            DocComment(s) => format!("///{}", s),

            Whitespace(s) | SimpleString(s) | SimpleCharacter(s) | SimpleFloat(s)
            | SimpleInteger(s) | SimpleSymbol(s) | SymbolLiteral(s) => s.clone(),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}
