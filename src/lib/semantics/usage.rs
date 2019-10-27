use crate::syntax::*;

#[derive(Debug)]
pub struct Usage {
    pub declaration: Node,
    pub references: Vec<Node>,
    pub import_directives: Vec<Node>,
}
