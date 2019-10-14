use crate::*;

pub struct Scope<'a> {
    parent: Option<&'a Scope<'a>>,

    declarations: HashMap<String, Id>,
}

impl<'a> Scope<'a> {
    pub fn new() -> Scope<'a> {
        Scope {
            parent: None,
            declarations: HashMap::new(),
        }
    }

    pub fn inner(&self) -> Scope {
        Scope {
            parent: Some(self),
            declarations: HashMap::new(),
        }
    }

    pub fn declare(&mut self, symbol: &syntax::Symbol) {
        self.declarations.insert(symbol.to_string(), symbol.id);
    }

    pub fn refer(&self, symbol: &syntax::Symbol) -> Option<Id> {
        let name = symbol.to_string();
        for (s, i) in self.declarations.iter() {
            if *s == name {
                return Some(*i);
            }
        }
        if let Some(ref parent) = self.parent {
            parent.refer(symbol)
        } else {
            None
        }
    }
}
