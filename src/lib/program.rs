use crate as loa;
use std::option::NoneError;
use std::process::Child;

pub fn new() -> Program {
    Program
}

pub struct Program;

impl Program {
    /// Sweep the entire program for all diagnostics,
    /// syntax errors and semantics.
    pub fn diagnostics(&mut self) -> loa::HashMap<loa::URI, Vec<loa::Diagnostic>> {
        loa::HashMap::new()
    }

    // SOURCE CODE MANIPULATION

    pub fn set(&mut self, uri: loa::URI, code: String) {}

    pub fn remove(&mut self, uri: loa::URI) {}

    pub fn edit(&mut self, edits: Vec<Edit>) {}

    // SEMANTIC QUERIES

    pub fn usage(&mut self, uri: loa::URI, at: (usize, usize)) -> Option<Usage> {
        None
    }

    pub fn completion(&mut self, uri: loa::URI, at: (usize, usize)) -> Option<Completion> {
        None
    }
}

pub struct Edit(
    pub loa::URI,
    pub ((usize, usize), (usize, usize)),
    pub String,
);

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
    pub name_span: loa::Span,
    pub node: loa::syntax::Node,
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
