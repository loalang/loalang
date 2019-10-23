use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Usage {
    pub declaration: Node,
    pub references: Vec<Node>,
}
