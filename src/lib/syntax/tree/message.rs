use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum Message {
    Unary(Id, Symbol),
    Binary(Id, Token, Expression),
    Keyword(Id, Keyworded<Expression>),
}

impl Message {
    pub fn selector(&self) -> String {
        match self {
            Message::Unary(_, ref s) => s.to_string(),
            Message::Binary(_, ref t, _) => t.lexeme(),
            Message::Keyword(_, ref kw) => kw
                .keywords
                .iter()
                .map(|(s, _, _)| s.to_string() + ":")
                .collect(),
        }
    }
}

impl Node for Message {
    fn id(&self) -> Option<Id> {
        match self {
            Message::Unary(ref id, _) => Some(*id),
            Message::Binary(ref id, _, _) => Some(*id),
            Message::Keyword(ref id, _) => Some(*id),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            Message::Unary(_, s) => s.span(),
            Message::Binary(_, op, pat) => match pat.span() {
                None => Some(op.span.clone()),
                Some(ref ps) => Some(op.span.through(ps)),
            },
            Message::Keyword(_, kw) => kw.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Message::Unary(_, s) => vec![s],
            Message::Binary(_, t, pp) => vec![t, pp],
            Message::Keyword(_, kw) => vec![kw],
        }
    }
}
