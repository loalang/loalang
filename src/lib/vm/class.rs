use crate::*;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<u64, Arc<Method>>,
    pub variables: HashMap<u64, Arc<Variable>>,
    pub variable_setters: HashMap<u64, Arc<Variable>>,
    pub variable_getters: HashMap<u64, Arc<Variable>>,
    pub offset: usize,
}

impl Class {
    pub fn new(name: String, offset: usize) -> Arc<Class> {
        Arc::new(Class {
            name,
            methods: HashMap::new(),
            variables: HashMap::new(),
            variable_setters: HashMap::new(),
            variable_getters: HashMap::new(),
            offset,
        })
    }
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Variable {
    pub name: String,
    pub id: u64,
    pub getter_id: u64,
    pub setter_id: u64,
}
