use crate::vm::*;
use crate::*;

pub struct Stack {
    vec: Vec<Arc<Object>>,
}

impl Stack {
    pub fn new() -> Stack {
        Stack { vec: vec![] }
    }

    pub fn push(&mut self, item: Arc<Object>) {
        self.vec.push(item);
    }

    pub fn pop(&mut self) -> Option<Arc<Object>> {
        self.vec.pop()
    }

    pub fn drop(&mut self, index: usize) {
        self.vec.remove(index);
    }

    pub fn at(&self, index: usize) -> Option<&Arc<Object>> {
        self.vec.get(self.vec.len() - 1 - index)
    }

    pub fn top(&self) -> Option<&Arc<Object>> {
        self.at(0)
    }

    pub fn size(&self) -> usize {
        self.vec.len()
    }
}

impl fmt::Debug for Stack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "STACK ------------------------")?;
        for (i, o) in self.vec.iter().enumerate() {
            writeln!(f, "{}: {}", i, o)?;
        }
        write!(f, "------------------------------")
    }
}
