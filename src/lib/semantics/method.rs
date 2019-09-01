use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub struct Method {
    pub visibility: Visibility,
    pub signature: Signature,
    pub implementation: Option<MethodImplementation>,
}

impl Method {
    pub fn selector(&self) -> &Symbol {
        &self.signature.selector
    }

    pub fn apply_type_arguments(&self, arguments: &Vec<(Arc<TypeParameter>, Type)>) -> Method {
        Method {
            visibility: self.visibility.clone(),
            signature: self.signature.apply_type_arguments(arguments),
            implementation: self.implementation.clone(),
        }
    }
}

#[derive(Clone)]
pub enum MethodImplementation {
    Body(Vec<Pattern>, Arc<Expression>),
    VariableGetter(*const Variable),
    VariableSetter(*const Variable),
}

#[derive(Clone)]
pub enum Visibility {
    Public,
    Private,
}
