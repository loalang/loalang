use crate::semantics::*;

pub struct Method {
    pub signature: Signature,
    pub implementation: Option<MethodImplementation>,
}

impl Method {
    pub fn selector(&self) -> &Symbol {
        &self.signature.selector
    }
}
