use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum MessagePattern {
    Unary(Id, Symbol),
    Binary(Id, Token, ParameterPattern),
    Keyword(Id, Keyworded<ParameterPattern>),
}

impl MessagePattern {
    pub fn selector(&self) -> String {
        match self {
            MessagePattern::Unary(_, ref s) => s.to_string(),
            MessagePattern::Binary(_, ref t, _) => t.lexeme(),
            MessagePattern::Keyword(_, ref kw) => kw
                .keywords
                .iter()
                .map(|(s, _, _)| s.to_string() + ":")
                .collect(),
        }
    }
}

impl Node for MessagePattern {
    fn id(&self) -> Option<Id> {
        match self {
            MessagePattern::Unary(ref id, _) => Some(*id),
            MessagePattern::Binary(ref id, _, _) => Some(*id),
            MessagePattern::Keyword(ref id, _) => Some(*id),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            MessagePattern::Unary(_, s) => s.span(),
            MessagePattern::Binary(_, op, pat) => match pat.span() {
                None => Some(op.span.clone()),
                Some(ref ps) => Some(op.span.through(ps)),
            },
            MessagePattern::Keyword(_, kw) => kw.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            MessagePattern::Unary(_, s) => vec![s],
            MessagePattern::Binary(_, t, pp) => vec![t, pp],
            MessagePattern::Keyword(_, kw) => vec![kw],
        }
    }
}
