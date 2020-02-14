use crate::vm::*;
use crate::*;

pub struct CallStack(Vec<Frame>);

pub enum Frame {
    Lazy(LazyFrame),
    Stack(StackFrame),
}

impl Frame {
    pub fn return_address(&self) -> usize {
        match self {
            Frame::Lazy(f) => f.return_address,
            Frame::Stack(f) => f.return_address,
        }
    }
}

pub struct LazyFrame {
    pub return_address: usize,
}

pub struct StackFrame {
    pub receiver: Arc<Object>,
    pub method: Arc<Method>,

    pub return_address: usize,
    pub callsite: SourceCodeLocation,
}

pub struct SourceCodeLocation(pub String, pub u64, pub u64);

impl CallStack {
    pub fn new() -> CallStack {
        CallStack(Vec::new())
    }

    pub fn push_lazy(&mut self, return_address: usize) {
        self.0.push(Frame::Lazy(LazyFrame { return_address }));
    }

    pub fn push(
        &mut self,
        receiver: Arc<Object>,
        method: Arc<Method>,
        return_address: usize,
        callsite: SourceCodeLocation,
    ) {
        self.0.push(Frame::Stack(StackFrame {
            receiver,
            method,
            return_address,
            callsite,
        }));
    }

    pub fn ret(&mut self) -> Option<usize> {
        Some(self.0.pop()?.return_address())
    }

    pub fn detach(&mut self) -> CallStack {
        std::mem::replace(self, CallStack::new())
    }
}

impl Into<Vec<Frame>> for CallStack {
    fn into(self) -> Vec<Frame> {
        self.0
    }
}

impl fmt::Debug for CallStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for frame in self.0.iter() {
            match frame {
                Frame::Stack(StackFrame {
                    receiver,
                    method,
                    callsite: SourceCodeLocation(uri, line, character),
                    ..
                }) => {
                    writeln!(
                        f,
                        "{} {}\n  ({}:{}:{})",
                        receiver
                            .class
                            .as_ref()
                            .map(|c| c.name.as_ref())
                            .unwrap_or("?"),
                        method.name,
                        uri,
                        line,
                        character
                    )?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
