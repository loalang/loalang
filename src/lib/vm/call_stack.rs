use crate::vm::*;
use crate::*;

#[derive(Clone)]
pub struct CallStack(Option<Arc<StackFrame>>);

#[derive(Clone)]
pub struct StackFrame {
    pub parent: Option<Arc<StackFrame>>,
    pub receiver: Arc<Object>,
    pub method: Arc<Method>,

    pub return_address: usize,
    pub callsite: SourceCodeLocation,
}

#[derive(Clone)]
pub struct SourceCodeLocation(pub String, pub u64, pub u64);

impl CallStack {
    pub fn new() -> CallStack {
        CallStack(None)
    }

    pub fn push(
        &mut self,
        receiver: Arc<Object>,
        method: Arc<Method>,
        return_address: usize,
        callsite: SourceCodeLocation,
    ) {
        let parent = self.0.clone();
        std::mem::replace(
            self,
            CallStack(Some(Arc::new(StackFrame {
                parent,
                receiver,
                method,
                return_address,
                callsite,
            }))),
        );
    }

    pub fn ret(&mut self) -> Option<usize> {
        let frame = self.0.as_ref()?;
        let return_address = frame.return_address;
        let parent = frame.parent.clone();
        std::mem::replace(self, CallStack(parent));
        Some(return_address)
    }

    pub fn detach(&mut self) -> CallStack {
        std::mem::replace(self, CallStack::new())
    }
}

impl Into<Vec<Arc<StackFrame>>> for CallStack {
    fn into(self) -> Vec<Arc<StackFrame>> {
        let mut frame = self.0;
        let mut frames = vec![];
        while let Some(f) = frame {
            frame = f.parent.clone();
            frames.push(f);
        }
        frames
    }
}

impl fmt::Debug for CallStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for frame in self.0.iter() {
            let StackFrame {
                receiver,
                method,
                callsite: SourceCodeLocation(uri, line, character),
                ..
            } = frame.as_ref();

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
        Ok(())
    }
}
