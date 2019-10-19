use crate::syntax::*;
use crate::*;
use std::option::NoneError;
use std::process::Child;
use std::ops::Deref;

pub struct Program {
    modules: HashMap<URI, (Arc<Source>, Arc<Tree>, LexicalReferences)>,
    syntax_errors: HashMap<URI, Vec<Diagnostic>>,
}

impl Program {
    pub fn new() -> Program {
        Program {
            modules: HashMap::new(),
            syntax_errors: HashMap::new(),
        }
    }

    pub fn diagnostics(&self) -> Vec<&Diagnostic> {
        self.syntax_errors
            .iter()
            .flat_map(|(_, d)| d.iter())
            .collect()
    }

    pub fn get(&self, uri: &URI) -> Option<&Arc<Source>> {
        Some(&self.modules.get(uri)?.0)
    }

    pub fn set(&mut self, source: Arc<Source>) {
        let uri = source.uri.clone();
        let (tree, diagnostics) = Parser::new(source.clone()).parse();
        self.modules.insert(
            uri.clone(),
            (source, tree.clone(), LexicalReferences::new(tree)),
        );
        self.syntax_errors.insert(uri, diagnostics);
    }

    pub fn replace(&mut self, uri: &URI, new_text: String) {
        self.set(Source::new(uri.clone(), new_text))
    }

    pub fn change(&mut self, span: Span, new_text: String) -> Result<(), NoneError> {
        let new_source = {
            let (source, _, _) = self.modules.get(&span.start.uri)?;

            let mut new_code = source.code.clone();
            new_code.replace_range(span.start.offset..span.end.offset, new_text.as_str());

            Source::new(source.uri.clone(), new_code)
        };

        self.set(new_source);

        Ok(())
    }

    pub fn definition(&mut self, location: Location) -> Option<Span> {
        let (_, t, refs) = self.modules.get_mut(&location.uri)?;
        let n = t.node_at(location)?;

        if let Symbol(_) = n.kind {
            if n.is_message_selector(t) {
                // Resolve message semantics and types and stuff
                None
            } else {
                refs.declaration_of(n).map(|n| n.span.clone())
            }
        } else {
            None
        }
    }
}

struct LexicalReferences {
    tree: Arc<Tree>,
    references: HashMap<Id, Id>,
}

impl LexicalReferences {
    pub fn new(tree: Arc<Tree>) -> LexicalReferences {
        LexicalReferences {
            tree,
            references: HashMap::new(),
        }
    }

    pub fn declaration_of(&mut self, node: &Node) -> Option<&Node> {
        if let Some(id) = self.references.get(&node.id).cloned() {
            if id == Id::NULL {
                None
            } else {
                self.tree.get(id)
            }
        } else if let Symbol(ref t) = node.kind {
            match self.find_declaration_of(node, t.lexeme().as_str()) {
                None => {
                    self.references.insert(node.id, Id::NULL);
                    None
                }
                Some(id) => {
                    self.references.insert(node.id, id);
                    self.tree.get(id)
                }
            }
        } else {
            None
        }
    }

    fn find_declaration_of(&self, node: &Node, name: &str) -> Option<Id> {
        if node.is_top_of_scope() {
            if let Some(id) = self.find_declaration_in_children(node, name) {
                return Some(id);
            }
        }

        if let Some(parent) = node.parent_id.and_then(|p| self.tree.get(p)) {
            self.find_declaration_of(parent, name)
        } else {
            None
        }
    }

    fn find_declaration_in_children(&self, node: &Node, name: &str) -> Option<Id> {
        for child in node.child_nodes(self.tree.as_ref()) {
            if let Some((id, n)) = child.as_declaration(self.tree.clone()) {
                if n == name {
                    return Some(id);
                }
            } else if child.is_top_of_scope() {
                continue;
            } else if let Some(id) = self.find_declaration_in_children(node, name) {
                return Some(id);
            }
        }
        None
    }
}

pub struct ChildrenIterator<'a> {
    tree: &'a Tree,
    id: Id,
    children: Option<Vec<ChildrenIterator<'a>>>,
}

impl<'a> ChildrenIterator<'a> {
    pub fn new(tree: &'a Tree, id: Id) -> ChildrenIterator<'a> {
        ChildrenIterator {
            tree,
            id,
            children: None,
        }
    }
}

impl<'a> Iterator for ChildrenIterator<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        if self.children.is_none() {
            self.children = Some(
                self.tree
                    .get(self.id)
                    .map(|n| {
                        n.children()
                            .into_iter()
                            .map(|c| ChildrenIterator::new(self.tree.clone(), c))
                            .collect()
                    })
                    .unwrap_or(vec![]),
            );
        }
        let mut children = self.children.as_mut().unwrap();
        let first = children.first_mut()?;
        match first.next() {
            None => {
                if children.len() == 0 {
                    None
                } else {
                    children.remove(0);
                    self.next()
                }
            },
            Some(n) => Some(n),
        }
    }
}
