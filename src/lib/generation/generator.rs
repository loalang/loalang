use crate::generation::*;
use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub type GenerationResult = Result<Instructions, GenerationError>;

pub struct Generator<'a> {
    analysis: &'a Analysis,
}

impl<'a> Generator<'a> {
    pub fn new(analysis: &'a Analysis) -> Generator<'a> {
        Generator { analysis }
    }

    pub fn generate(&self, uri: &URI) -> GenerationResult {
        let root = self.analysis.navigator.root_of(uri)?;
        match root.kind {
            Module { .. } => self.generate_module(&root),
            REPLLine { .. } => self.generate_repl_line(&root),
            _ => Err(invalid_node(&root, "Module or REPLLine expected.")),
        }
    }

    pub fn generate_module(&self, module: &Node) -> GenerationResult {
        let mut instructions = Instructions::new();
        if let Module {
            ref module_declarations,
            ..
        } = module.kind
        {
            for dec in module_declarations {
                let dec = self.analysis.navigator.find_child(module, *dec)?;
                instructions.extend(self.generate_module_declaration(&dec)?);
            }
        }
        Ok(instructions)
    }

    pub fn generate_module_declaration(&self, module_declaration: &Node) -> GenerationResult {
        match module_declaration.kind {
            Exported(_, declaration) => {
                let declaration = self
                    .analysis
                    .navigator
                    .find_child(&module_declaration, declaration)?;
                self.generate_declaration(&declaration)
            }
            _ => self.generate_declaration(&module_declaration),
        }
    }

    pub fn generate_repl_line(&self, repl_line: &Node) -> GenerationResult {
        match repl_line.kind {
            REPLLine { ref statements } => {
                let mut instructions = Instructions::new();
                for statement in statements.iter() {
                    let statement = self.analysis.navigator.find_child(repl_line, *statement)?;
                    instructions.extend(self.generate_repl_statement(&statement)?);
                }
                Ok(instructions)
            }
            _ => Err(invalid_node(repl_line, "REPLLine expected.")),
        }
    }

    pub fn generate_repl_statement(&self, repl_statement: &Node) -> GenerationResult {
        match repl_statement.kind {
            REPLExpression { expression, .. } => {
                let expression = self
                    .analysis
                    .navigator
                    .find_child(&repl_statement, expression)?;
                self.generate_expression(&expression)
            }
            _ => self.generate_declaration(&repl_statement),
        }
    }

    pub fn generate_expression(&self, expression: &Node) -> GenerationResult {
        info!("GENERATE EXPRESSION: {:?}", expression);
        Ok(vec![].into())
    }

    pub fn generate_declaration(&self, declaration: &Node) -> GenerationResult {
        info!("GENERATE DECLARATION: {:?}", declaration);
        Ok(vec![].into())
    }
}

fn invalid_node(node: &Node, message: &str) -> GenerationError {
    GenerationError::InvalidNode(node.clone(), message.into())
}
