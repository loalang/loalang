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

    pub fn check(&mut self) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        for checker in checkers::checkers().iter() {
            checker.check(self, &mut diagnostics);
        }

        diagnostics
    }

    pub fn usage(&mut self, node: &syntax::Node) -> Option<Arc<Usage>> {
        let tree = self.modules.get(&node.span.start.uri)?.clone();
        let modules = self.modules.clone();
        if let syntax::Symbol(ref t) = node.kind {
            self.usage
                .cache(node.id, |cache| {
                    let name = t.lexeme();
                    let node = tree.get(node.parent_id?)?;
                    let usage = Arc::new(find_usage(modules, tree, node.clone(), name)?);

                    cache.set(usage.declaration.id, Some(usage.clone()));
                    for n in usage.references.iter() {
                        cache.set(n.id, Some(usage.clone()));
                    }
                    Some(usage)
                })
                .clone()
        } else {
            None
        }
    }

    pub fn all_reference_symbols(&self) -> Vec<syntax::Node> {
        let mut references = vec![];
        for (_, tree) in self.modules.iter() {
            if let Some(root) = tree.root() {
                references.extend(
                    root.all_references_downwards(tree.clone())
                        .into_iter()
                        .filter_map(|n| tree.get(n.symbol_id()?)),
                );
            }
        }
        references
    }

    pub fn all_references(&self) -> Vec<syntax::Node> {
        let mut references = vec![];
        for (_, tree) in self.modules.iter() {
            if let Some(root) = tree.root() {
                references.extend(root.all_references_downwards(tree.clone()));
            }
        }
        references
    }
}

fn find_usage(
    modules: Arc<HashMap<URI, Arc<syntax::Tree>>>,
    tree: Arc<syntax::Tree>,
    from: syntax::Node,
    name: String,
) -> Option<semantics::Usage> {
    if from.is_declaration() || from.is_import_directive() {
        Some(semantics::Usage {
            declaration: from.clone(),
            references: find_references(tree, from, name),
        })
    } else if from.is_reference() {
        find_declaration(modules.clone(), tree.clone(), from, name.clone())
            .and_then(|d| find_usage(modules, tree, d, name))
    } else {
        None
    }
}

fn find_declaration(
    modules: Arc<HashMap<URI, Arc<syntax::Tree>>>,
    tree: Arc<syntax::Tree>,
    from: syntax::Node,
    name: String,
) -> Option<syntax::Node> {
    match from.closest_scope_root_upwards(tree.clone()) {
        None => None,
        Some(scope_root) => {
            let mut result = None;
            scope_root.traverse(tree.clone(), &mut |node| {
                // We do not traverse down scope roots, since
                // declarations declared there is not reachable
                // to the original reference.
                if node.id != scope_root.id && node.is_scope_root() {
                    return false;
                }

                if node.is_import_directive() {
                    if let syntax::ImportDirective {
                        qualified_symbol,
                        symbol,
                        ..
                    } = node.kind
                    {
                        if let Some(qualified_symbol) = tree.get(qualified_symbol) {
                            if let syntax::QualifiedSymbol { symbols, .. } = qualified_symbol.kind {
                                if let Some(mut imported_symbol) = symbols.last().cloned() {
                                    if symbol != Id::NULL {
                                        imported_symbol = symbol;
                                    }
                                    if let Some(imported_symbol) = tree.get(imported_symbol) {
                                        if let syntax::Symbol(t) = imported_symbol.kind {
                                            if t.lexeme() == name {
                                                match find_declaration_from_import(
                                                    node,
                                                    modules.clone(),
                                                ) {
                                                    Some(n) => {
                                                        result = Some(n);
                                                    }
                                                    None => {
                                                        result = Some(node.clone());
                                                    }
                                                }
                                                return false;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if node.is_declaration() {
                    if let Some(symbol) = node.symbol_id().and_then(|id| tree.get(id)) {
                        if let syntax::Symbol(ref t) = symbol.kind {
                            if t.lexeme() == name {
                                result = Some(node.clone());
                                return false;
                            }
                        }
                    }
                }

                true
            });
            if result.is_some() {
                return result;
            }
            let parent = tree.get(scope_root.parent_id?)?;
            find_declaration(modules, tree, parent, name)
        }
    }
}

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

fn find_references(
    tree: Arc<syntax::Tree>,
    declaration: syntax::Node,
    name: String,
) -> Vec<syntax::Node> {
    match declaration.closest_scope_root_upwards(tree.clone()) {
        None => vec![],
        Some(scope_root) => scope_root.all_downwards(tree.clone(), &|n| {
            if !n.is_reference() {
                return false;
            }

            n.symbol_id()
                .and_then(|id| tree.get(id))
                .and_then(|s| {
                    if let syntax::Symbol(ref t) = s.kind {
                        if t.lexeme() == name {
                            Some(true)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or(false)
        }),
    }
}
