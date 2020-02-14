use crate::*;

pub struct Stack<T> {
    vec: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack { vec: vec![] }
    }

    pub fn push(&mut self, item: T) {
        self.vec.push(item);
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
        self.vec.extend(items);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.vec.pop()
    }

    pub fn drop(&mut self, index: usize) {
        self.vec.remove(self.vec.len() - 1 - index);
    }

    pub fn at(&self, index: usize) -> Option<&T> {
        self.vec.get(self.vec.len() - 1 - index)
    }

    pub fn top(&self) -> Option<&T> {
        self.at(0)
    }

    pub fn size(&self) -> usize {
        self.vec.len()
    }

    pub fn iter(&self) -> std::iter::Rev<std::slice::Iter<T>> {
        self.vec.iter().rev()
    }
}

impl<T: fmt::Display> fmt::Debug for Stack<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "STACK ------------------------")?;
        for (i, o) in self.vec.iter().rev().enumerate() {
            writeln!(f, "{}: {}", i, o)?;
        }
        write!(f, "------------------------------")
    }
}
