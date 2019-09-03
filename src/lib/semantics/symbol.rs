use crate::*;

#[derive(Clone)]
pub struct Symbol(pub Option<Span>, pub String);

impl std::hash::Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let Symbol(_, s) = self;
        s.hash(state);
    }
}

impl Eq for Symbol {}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        let Symbol(_, lhs) = self;
        let Symbol(_, rhs) = other;
        lhs == rhs
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.1)
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{}", self.1)
    }
}
