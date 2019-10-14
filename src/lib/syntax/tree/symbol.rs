use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Symbol {
    pub id: Id,
    pub token: Token,
}

impl ToString for Symbol {
    fn to_string(&self) -> String {
        self.token.lexeme()
    }
}

impl Node for Symbol {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        self.token.span()
    }

    fn children(&self) -> Vec<&dyn Node> {
        vec![&self.token]
    }
}
