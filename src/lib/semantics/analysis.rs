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

        let mut declarations = vec![];

        while let Some(scope_root) = navigator.closest_scope_root_upwards(&from) {
            navigator.traverse(&scope_root, &mut |n| {
                // Don't traverse into lower scopes.
                if n.is_scope_root() && n.id != scope_root.id {
                    // Classes exist outside their own scope, though.
                    if n.is_class() {
                        if let Some((name, _)) = navigator.symbol_of(&n) {
                            declarations.push((name, n.clone()));
                        }
                    }

                    return false;
                }

                if n.is_declaration(kind) {
                    if let Some((name, _)) = navigator.symbol_of(&n) {
                        declarations.push((name, n.clone()));
                    }
                }

                if n.is_import_directive() {
                    if let Some((name, _)) = navigator.symbol_of(&n) {
                        if let Some(n) = navigator.find_declaration_from_import(&n) {
                            declarations.push((name, n.clone()));
                        }
                    }
                }

                true
            });

            if let Some(parent) = scope_root
                .parent_id
                .and_then(|pid| navigator.find_node_in(&uri, pid))
            {
                from = parent;
            } else {
                break;
            }
        }

        declarations
    }
}
