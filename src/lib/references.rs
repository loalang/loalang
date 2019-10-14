use crate::*;

#[derive(Clone, Debug)]
pub struct References {
    references: HashMap<Id, Id>,
}

impl References {
    pub fn new() -> References {
        References {
            references: HashMap::new(),
        }
    }

    pub fn declaration_of(&self, reference: Id) -> Option<Id> {
        self.references.get(&reference).cloned()
    }

    pub fn references_of(&self, declaration: Id) -> Vec<Id> {
        let mut references = vec![];

        for (r, d) in self.references.iter() {
            if *d == declaration {
                references.push(*r);
            }
        }

        references
    }

    pub fn register_reference(&mut self, from: Id, to: Id) {
        self.references.insert(from, to);
    }
}
