use crate::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct Id(usize);

static NODE_GEN: AtomicUsize = AtomicUsize::new(0xffff);

impl Id {
    pub fn new() -> Id {
        Id(NODE_GEN.fetch_add(1, Ordering::SeqCst))
    }

    pub const NULL: Id = Id(0);
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Id(value) = self;
        write!(f, "#{:X}", value)
    }
}
