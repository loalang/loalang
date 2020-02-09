use crate::vm::*;
use crate::*;

#[derive(Clone, Debug)]
pub struct CallStack(Vec<StackFrame>);

#[derive(Clone, Debug)]
pub struct StackFrame {
    pub receiver: Arc<Object>,
    pub method: Arc<Method>,

    pub return_address: usize,
    pub callsite: SourceCodeLocation,
}

#[derive(Clone, Debug)]
pub struct SourceCodeLocation(pub String, pub u64, pub u64);

impl CallStack {
    pub fn new() -> CallStack {
        CallStack(Vec::new())
    }

    pub fn push(
        &mut self,
        receiver: Arc<Object>,
        method: Arc<Method>,
        return_address: usize,
        callsite: SourceCodeLocation,
    ) {
        self.0.push(StackFrame {
            receiver,
            method,
            return_address,
            callsite,
        });
    }

    pub fn ret(&mut self) -> Option<usize> {
        Some(self.0.pop()?.return_address)
    }
}

impl Into<Vec<StackFrame>> for CallStack {
    fn into(self) -> Vec<StackFrame> {
        self.0
    }
}
