use crate::syntax::*;
use crate::*;

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
                return Some(self.get(*import_directives.last()?)?.span.end)
            }

            Some(self.get(namespace_directive)?.span.end)
        } else {
            None
        }
    }
}
