use crate::*;
use crate::semantics::*;

#[derive(Clone)]
pub struct Signature {
    pub selector: Symbol,
    pub type_parameters: Vec<Arc<TypeParameter>>,
    pub parameters: Vec<Type>,
    pub return_type: Type,
}

impl Signature {
    pub fn apply_type_arguments(&self, arguments: &Vec<(Arc<TypeParameter>, Type)>) -> Signature {
        Signature {
            selector: self.selector.clone(),
            type_parameters: self.type_parameters.clone(),
            parameters: self.parameters.iter().map(|t| t.apply_type_arguments(arguments)).collect(),
            return_type: self.return_type.apply_type_arguments(arguments),
        }
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} -> {}", self.selector, self.return_type)
    }
}

