use crate::*;

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<u64, Arc<Method>>,
    pub offset: usize,
}

impl Class {
    pub fn new(name: String, offset: usize) -> Arc<Class> {
        Arc::new(Class {
            name,
            methods: HashMap::new(),
            offset,
        })
    }
}

#[derive(Debug)]
pub struct Method {
    pub name: String,
    pub offset: usize,
}
