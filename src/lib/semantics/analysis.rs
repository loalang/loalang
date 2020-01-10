use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub struct Analysis {
    pub types: Types,
    pub navigator: Navigator,
}

impl Analysis {
    pub fn new(modules: Arc<HashMap<URI, Arc<syntax::Tree>>>) -> Analysis {
        let navigator = Navigator::new(modules);
        let types = Types::new(navigator.clone());

        Analysis { navigator, types }
    }

    pub fn check(&mut self) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for checker in checkers::checkers().iter() {
            checker.check(self, &mut diagnostics);
        }

        diagnostics
    }
}

impl<I: Iterator<Item = (URI, Arc<syntax::Tree>)>> From<I> for Analysis {
    fn from(iterator: I) -> Self {
        Self::new(Arc::new(iterator.collect()))
    }
}
