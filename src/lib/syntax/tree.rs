use crate::syntax::*;
use crate::*;

pub struct Tree {
    nodes: HashMap<Id, Node>,
    root: Id,
}

impl Tree {
    pub fn new() -> Tree {
        Tree {
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
}
