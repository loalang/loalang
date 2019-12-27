use crate::semantics::*;
use crate::syntax::DeclarationKind;
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

    pub fn declarations_in_scope(
        &self,
        mut from: syntax::Node,
        kind: DeclarationKind,
    ) -> Vec<(String, syntax::Node)> {
        let uri = from.span.start.uri.clone();
        let navigator = &self.navigator;

        let mut declarations = HashMap::new();

        while let Some(scope_root) = navigator.closest_scope_root_upwards(&from) {
            let mut traverse = |n: &syntax::Node| {
                if n.is_declaration(kind) {
                    if let Some((name, _)) = navigator.symbol_of(&n) {
                        declarations.insert(name, n.clone());
                    }
                }

                if n.is_import_directive() {
                    if let Some((name, _)) = navigator.symbol_of(&n) {
                        if let Some(n) = navigator.find_declaration_from_import(&n) {
                            declarations.insert(name, n.clone());
                        }
                    }
                }

                // Don't traverse into lower scopes.
                n.id == scope_root.id || !n.is_scope_root() || n.is_repl_line()
            };
            if scope_root.is_repl_line() {
                navigator.traverse_all_repl_lines(&mut traverse);
            } else {
                navigator.traverse(&scope_root, &mut traverse);
            }

            if let Some(parent) = scope_root
                .parent_id
                .and_then(|pid| navigator.find_node_in(&uri, pid))
            {
                from = parent;
            } else {
                break;
            }
        }

        declarations.into_iter().collect()
    }
}

impl<I: Iterator<Item = (URI, Arc<syntax::Tree>)>> From<I> for Analysis {
    fn from(iterator: I) -> Self {
        Self::new(Arc::new(iterator.collect()))
    }
}
