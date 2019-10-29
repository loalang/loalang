use crate::semantics::*;
use crate::syntax::DeclarationKind;
use crate::*;

pub struct Analysis {
    modules: Arc<HashMap<URI, Arc<syntax::Tree>>>,
    usage: Cache<Id, Option<Arc<Usage>>>,
    pub types: Types,
}

impl Analysis {
    pub fn new(modules: Arc<HashMap<URI, Arc<syntax::Tree>>>) -> Analysis {
        Analysis {
            modules: modules.clone(),
            usage: Cache::new(),
            types: Types::new(ProgramNavigator::new(modules)),
        }
    }

    pub fn navigator(&self) -> ProgramNavigator {
        ProgramNavigator::new(self.modules.clone())
    }

    pub fn check(&mut self) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for checker in checkers::checkers().iter() {
            checker.check(self, &mut diagnostics);
        }

        diagnostics
    }

    pub fn usage(&mut self, node: &syntax::Node) -> Option<Arc<Usage>> {
        let navigator = self.navigator();
        let types = self.types.clone();
        self.usage
            .cache(node.id, move |cache| {
                let usage = navigator.find_usage(node, DeclarationKind::Any, &types)?;

                cache.set(usage.declaration.id, Some(usage.clone()));
                for n in usage.references.iter() {
                    cache.set(n.id, Some(usage.clone()));
                }
                for n in usage.import_directives.iter() {
                    cache.set(n.id, Some(usage.clone()));
                }
                Some(usage)
            })
            .clone()
    }

    pub fn all_reference_symbols(&self, kind: DeclarationKind) -> Vec<syntax::Node> {
        self.navigator().all_reference_symbols(kind)
    }

    pub fn all_references(&self, kind: DeclarationKind) -> Vec<syntax::Node> {
        self.navigator().all_references(kind)
    }

    pub fn declaration_is_exported(&self, declaration: &syntax::Node) -> bool {
        self.navigator().declaration_is_exported(declaration)
    }

    pub fn declarations_in_scope(
        &self,
        mut from: syntax::Node,
        kind: DeclarationKind,
    ) -> Vec<(String, syntax::Node)> {
        let uri = from.span.start.uri.clone();
        let navigator = self.navigator();

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
