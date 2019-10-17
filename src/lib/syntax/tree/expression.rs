use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum Expression {
    Reference(Id, Symbol),
    MessageSend(Id, Box<Expression>, Box<Message>),
}

impl Node for Expression {
    fn id(&self) -> Option<Id> {
        match self {
            Expression::Reference(id, _) => Some(*id),
            Expression::MessageSend(id, _, _) => Some(*id),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            Expression::Reference(_, s) => s.span(),
            Expression::MessageSend(_, e, m) => {
                let first_span: Span;
                let last_span: Span;

                if let Some(s) = e.span() {
                    first_span = s;
                } else if let Some(s) = m.span() {
                    first_span = s;
                } else {
                    return None;
                }

                if let Some(s) = m.span() {
                    last_span = s;
                } else if let Some(s) = e.span() {
                    last_span = s;
                } else {
                    last_span = first_span.clone();
                }

                Some(Span::over(first_span, last_span))
            }
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            Expression::Reference(_, s) => vec![s],
            Expression::MessageSend(_, e, m) => vec![e.as_ref(), m.as_ref()],
        }
    }
}
