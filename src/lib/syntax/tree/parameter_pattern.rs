use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum ParameterPattern {
    Nothing(Id, Token),
    Parameter(Id, Option<TypeExpression>, Option<Symbol>),
}

impl Node for ParameterPattern {
    fn id(&self) -> Option<Id> {
        match self {
            ParameterPattern::Nothing(i, _) => Some(*i),
            ParameterPattern::Parameter(i, _, _) => Some(*i),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            ParameterPattern::Nothing(_, t) => Some(t.span.clone()),
            ParameterPattern::Parameter(_, Some(t), Some(s)) => match t.span() {
                None => Some(s.token.span.clone()),
                Some(ss) => Some(ss.through(&s.token.span)),
            },
            ParameterPattern::Parameter(_, None, Some(s)) => s.span(),
            ParameterPattern::Parameter(_, Some(t), None) => t.span(),
            ParameterPattern::Parameter(_, None, None) => None,
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ParameterPattern::Nothing(_, t) => vec![t],
            ParameterPattern::Parameter(_, t, s) => {
                let mut children: Vec<&dyn Node> = vec![];

                push!(children, t);
                push!(children, s);

                children
            }
        }
    }
}
