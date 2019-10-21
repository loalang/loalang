use crate::*;

mod module_cell;
pub use self::module_cell::*;

mod server;
pub use self::server::*;

pub struct Usage {
    pub declaration: NamedNode,
    pub references: Vec<NamedNode>,
}

impl Usage {
    pub fn named_nodes(self) -> Vec<NamedNode> {
        let mut nodes = self.references;
        nodes.push(self.declaration);
        nodes
    }
}

pub struct NamedNode {
    pub name: String,
    pub name_span: Span,
    pub node: syntax::Node,
}

pub enum Completion {
    MessageSends(String, Vec<MessageSignature>),
    VariablesInScope(Vec<Variable>),
}

pub struct Variable {
    pub name: String,
    pub type_: Type,
}

pub enum MessageSignature {
    Unary(String, Type),
    Binary((String, Type), Type),
    Keyword(Vec<(String, Type)>, Type),
}

pub enum Type {
    Named(String, Vec<Type>),
    Tuple(Vec<Type>),
}
