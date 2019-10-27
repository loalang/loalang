use crate::semantics::*;
use crate::*;

pub struct Analysis {
    modules: Arc<HashMap<URI, Arc<syntax::Tree>>>,
    usage: Cache<Id, Option<Arc<Usage>>>,
}

impl Analysis {
    pub fn new(modules: Arc<HashMap<URI, Arc<syntax::Tree>>>) -> Analysis {
        Analysis {
            modules,
            usage: Cache::new(),
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
        self.usage
            .cache(node.id, move |cache| {
                let usage = navigator.find_usage(node)?;

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

    pub fn all_reference_symbols(&self) -> Vec<syntax::Node> {
        self.navigator().all_reference_symbols()
    }

    pub fn all_references(&self) -> Vec<syntax::Node> {
        self.navigator().all_references()
    }

    pub fn declaration_is_exported(&self, declaration: &syntax::Node) -> bool {
        self.navigator().declaration_is_exported(declaration)
    }
}

/*
fn find_declaration_from_import(
    import_directive: &syntax::Node,
    modules: Arc<HashMap<URI, Arc<syntax::Tree>>>,
) -> Option<syntax::Node> {
    if let syntax::ImportDirective {
        qualified_symbol, ..
    } = import_directive.kind
    {
        let (tree, qualified_symbol) = find_node_in_modules(modules.as_ref(), qualified_symbol)?;
        if let syntax::QualifiedSymbol { mut symbols } = qualified_symbol.kind {
            let declaration_symbol = find_node_in_modules(modules.as_ref(), symbols.pop()?)
                .map(|(_, n)| n)
                .and_then(symbol_to_string)?;
            let namespace = qualified_symbol_to_string(&tree, symbols);

            for (_uri, other_tree) in modules_with_namespace(modules.clone(), namespace) {
                if Arc::ptr_eq(&tree, &other_tree) {
                    continue;
                }

                if let Some(root) = other_tree.root().cloned() {
                    if let Some(n) = find_declaration(
                        modules.clone(),
                        other_tree,
                        root,
                        declaration_symbol.clone(),
                    ) {
                        return Some(n);
                    }
                }
            }
        }
    }
    None
}

fn modules_with_namespace(
    modules: Arc<HashMap<URI, Arc<syntax::Tree>>>,
    namespace: String,
) -> Vec<(URI, Arc<syntax::Tree>)> {
    modules
        .iter()
        .map(|(a, b)| (a.clone(), b.clone()))
        .filter(|(_, tree)| namespace_from_tree(tree) == Some(namespace.clone()))
        .collect()
}

fn namespace_from_tree(tree: &syntax::Tree) -> Option<String> {
    let module = tree.root()?;
    if let syntax::Module {
        namespace_directive,
        ..
    } = module.kind
    {
        if let syntax::NamespaceDirective {
            qualified_symbol, ..
        } = tree.get(namespace_directive)?.kind
        {
            if let syntax::QualifiedSymbol { symbols } = tree.get(qualified_symbol)?.kind {
                return Some(qualified_symbol_to_string(tree, symbols));
            }
        }
    }
    None
}

fn qualified_symbol_to_string(tree: &syntax::Tree, symbols: Vec<Id>) -> String {
    symbols
        .into_iter()
        .filter_map(|id| tree.get(id))
        .filter_map(symbol_to_string)
        .collect::<Vec<_>>()
        .join("/")
}

fn symbol_to_string(node: syntax::Node) -> Option<String> {
    match node.kind {
        syntax::Symbol(ref t) => Some(t.lexeme()),
        _ => None,
    }
}

fn find_node_in_modules(
    modules: &HashMap<URI, Arc<syntax::Tree>>,
    id: Id,
) -> Option<(Arc<syntax::Tree>, syntax::Node)> {
    for (_, tree) in modules.iter() {
        if let Some(node) = tree.get(id) {
            return Some((tree.clone(), node));
        }
    }
    None
}

*/
