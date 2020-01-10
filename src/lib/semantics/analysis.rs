use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub struct Analysis {
    pub types: Types,
    pub navigator: Navigator,

    diagnostics_cache: Option<Vec<Diagnostic>>,
}

impl Analysis {
    pub fn new(modules: Arc<HashMap<URI, Arc<syntax::Tree>>>) -> Analysis {
        let navigator = Navigator::new(modules);
        let types = Types::new(navigator.clone());

        Analysis {
            navigator,
            types,
            diagnostics_cache: None,
        }
    }

    pub fn check(&mut self) -> &Vec<Diagnostic> {
        if self.diagnostics_cache.is_none() {
            let mut diagnostics = vec![];

            for checker in checkers::checkers().iter() {
                checker.check(self, &mut diagnostics);
            }

            self.diagnostics_cache = Some(diagnostics);
        }
        self.diagnostics_cache.as_ref().unwrap()
    }
}

impl<I: Iterator<Item = (URI, Arc<syntax::Tree>)>> From<I> for Analysis {
    fn from(iterator: I) -> Self {
        Self::new(Arc::new(iterator.collect()))
    }
}
