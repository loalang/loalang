use crate::semantics::*;
use crate::*;

mod module_cell;
pub use self::module_cell::*;

mod server;
pub use self::server::*;

#[derive(Debug)]
pub struct Usage {
    pub handle: NamedNode,
    pub declaration: NamedNode,
    pub references: Vec<NamedNode>,
    pub imports: Vec<NamedNode>,
}

impl Usage {
    pub fn named_nodes(&self) -> Vec<NamedNode> {
        let mut nodes = self.references.clone();
        nodes.push(self.declaration.clone());
        nodes.extend(self.imports.iter().cloned());
        nodes
    }

    pub fn handle_is_aliased(&self) -> bool {
        self.handle.name != self.declaration.name
    }
}

#[derive(Debug, Clone)]
pub struct NamedNode {
    pub name: String,
    pub name_span: Span,
    pub node: syntax::Node,
}

pub enum Completion {
    Behaviours(Vec<Behaviour>),
    VariablesInScope(Vec<Variable>),
}

pub struct Variable {
    pub name: String,
    pub type_: Type,
    pub kind: VariableKind,
}

pub enum VariableKind {
    Unknown,
    Class,
    Parameter,
}
