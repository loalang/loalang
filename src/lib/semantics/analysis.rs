use crate::semantics::*;
use crate::*;

pub struct Analysis {
    modules: HashMap<URI, Arc<syntax::Tree>>,
    usage: Cache<Id, Option<Arc<Usage>>>,
}

impl Analysis {
    pub fn new(modules: HashMap<URI, Arc<syntax::Tree>>) -> Analysis {
        Analysis {
            modules,
            usage: Cache::new(),
        }
    }

    pub fn usage(&mut self, node: &syntax::Node) -> Option<Arc<Usage>> {
        let tree = self.modules.get(&node.span.start.uri)?.clone();
        if let syntax::Symbol(ref t) = node.kind {
            self.usage
                .cache(node.id, |cache| {
                    let name = t.lexeme();
                    let node = tree.get(node.parent_id?)?;
                    let usage = Arc::new(find_usage(tree, node.clone(), name)?);

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
}

fn find_usage(
    tree: Arc<syntax::Tree>,
    from: syntax::Node,
    name: String,
) -> Option<semantics::Usage> {
    if from.is_declaration() {
        Some(semantics::Usage {
            declaration: from.clone(),
            references: find_references(tree, from, name),
        })
    } else if from.is_reference() {
        find_declaration(tree.clone(), from, name.clone()).and_then(|d| find_usage(tree, d, name))
    } else {
        None
    }
}

fn find_declaration(
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
            find_declaration(tree, parent, name)
        }
    }
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
