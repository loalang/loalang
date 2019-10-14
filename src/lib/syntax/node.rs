use crate::syntax::*;
use crate::*;
use std::any::Any;

pub trait Node: fmt::Debug {
    fn id(&self) -> Option<Id>;
    fn as_any(&self) -> &dyn Any;
    fn span(&self) -> Option<Span>;
    fn children(&self) -> Vec<&dyn Node>;
    fn contains_location(&self, location: &Location) -> bool {
        match self.span() {
            None => false,
            Some(s) => s.contains_location(location),
        }
    }
    fn is_token(&self) -> bool {
        false
    }
}

pub fn traverse<'b>(node: &'b dyn Node) -> NodeTraverser<'b> {
    NodeTraverser {
        nodes: vec![node],
        current: None,
    }
}

pub fn cast<T: 'static>(node: &dyn Node) -> Option<&T> {
    node.as_any().downcast_ref::<T>()
}

pub struct NodeTraverser<'a> {
    nodes: Vec<&'a dyn Node>,
    current: Option<Box<NodeTraverser<'a>>>,
}

impl<'a> Iterator for NodeTraverser<'a> {
    type Item = &'a dyn Node;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current {
            None => {
                if self.nodes.len() == 0 {
                    return None;
                }
                let n = self.nodes.remove(0);
                self.current = Some(Box::new(NodeTraverser {
                    nodes: n.children(),
                    current: None,
                }));

                return Some(n);
            }

            Some(ref mut current) => match current.next() {
                None => {
                    self.current = None;
                    self.next()
                }

                Some(i) => Some(i),
            },
        }
    }
}

impl Node for Token {
    fn id(&self) -> Option<Id> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        Some(self.span.clone())
    }

    fn children(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn is_token(&self) -> bool {
        true
    }
}
