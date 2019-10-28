use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Tree {
    source: Arc<Source>,
    nodes: HashMap<Id, Node>,
    root: Id,
}

impl Tree {
    pub fn new(source: Arc<Source>) -> Tree {
        Tree {
            source,
            nodes: HashMap::new(),
            root: Id::NULL,
        }
    }

    pub fn get(&self, id: Id) -> Option<Node> {
        self.nodes.get(&id).cloned()
    }

    pub fn get_mut(&mut self, id: Id) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    pub fn add(&mut self, node: Node) {
        let id = node.id;
        if node.parent_id.is_none() {
            self.root = id;
        }
        self.nodes.insert(id, node);
    }

    pub fn root(&self) -> Option<&Node> {
        self.nodes.get(&self.root)
    }

    pub fn nodes_around(
        &self,
        location: Location,
    ) -> (Option<&Node>, Option<&Node>, Option<&Node>) {
        let mut nodes_before = vec![];
        let mut nodes_at = vec![];
        let mut nodes_after = vec![];

        for node in self.nodes.values() {
            let leaves = node.leaves();
            if node.span.end <= location {
                nodes_before.push((node.span.clone(), node))
            } else if node.span.start >= location {
                nodes_after.push((node.span.clone(), node))
            } else {
                nodes_at.push((node.span.clone(), node))
            }
            for leaf in leaves {
                if leaf.span.end <= location {
                    nodes_before.push((leaf.span.clone(), node))
                } else if leaf.span.start >= location {
                    nodes_after.push((leaf.span.clone(), node))
                } else {
                    nodes_at.push((leaf.span.clone(), node))
                }
            }
        }

        nodes_before.sort_by(|(a, _), (b, _)| b.end.cmp(&a.end).then(a.len().cmp(&b.len())));

        nodes_at.sort_by(|(a, _), (b, _)| b.start.cmp(&a.start).then(a.len().cmp(&b.len())));

        nodes_after.sort_by(|(a, _), (b, _)| a.start.cmp(&b.start).then(a.len().cmp(&b.len())));

        (
            nodes_before.first().map(|(_, n)| *n),
            nodes_at.first().map(|(_, n)| *n),
            nodes_after.first().map(|(_, n)| *n),
        )
    }

    pub fn node_at(&self, location: Location) -> Option<&Node> {
        let mut current_node = self.root;
        'children: loop {
            if let Some(node) = self.nodes.get(&current_node) {
                if node.span.contains_location(&location) {
                    for child in node
                        .children()
                        .into_iter()
                        .filter_map(|child| self.nodes.get(&child))
                    {
                        if child.span.contains_location(&location) {
                            current_node = child.id;
                            continue 'children;
                        }
                    }
                }
            }
            break;
        }
        self.nodes.get(&current_node)
    }

    pub fn namespace(&self) -> Option<String> {
        let mut segments = vec![];

        let root = self.get(self.root)?;
        if let Module {
            namespace_directive,
            ..
        } = root.kind
        {
            let namespace_directive = self.get(namespace_directive)?;
            if let NamespaceDirective {
                qualified_symbol, ..
            } = namespace_directive.kind
            {
                let qualified_symbol = self.get(qualified_symbol)?;
                if let QualifiedSymbol { symbols } = qualified_symbol.kind {
                    for symbol in symbols {
                        let symbol = self.get(symbol)?;
                        if let Symbol(t) = symbol.kind {
                            segments.push(t.lexeme());
                        }
                    }
                }
            }
        }

        if segments.len() == 0 {
            None
        } else {
            Some(segments.join("/"))
        }
    }

    pub fn end_of_import_list_location(&self) -> Location {
        self.end_of_import_list_location_impl()
            .unwrap_or_else(|| Location::at_offset(&self.source, 0))
    }

    fn end_of_import_list_location_impl(&self) -> Option<Location> {
        if let Module {
            namespace_directive,
            import_directives,
            ..
        } = self.get(self.root)?.kind
        {
            if import_directives.len() > 0 {
                return Some(self.get(*import_directives.last()?)?.span.end);
            }

            Some(self.get(namespace_directive)?.span.end)
        } else {
            None
        }
    }
}
