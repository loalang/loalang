use crate::generation::Instruction;
use crate::*;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<u64, Arc<Method>>,
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub instructions: Vec<Instruction>,
}
