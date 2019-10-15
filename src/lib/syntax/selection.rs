use crate::syntax::*;
use crate::*;

pub struct Selection<'a> {
    nodes: Vec<&'a dyn Node>,
}

impl<'a> Selection<'a> {
    pub fn new(mut nodes: Vec<&'a dyn Node>) -> Selection<'a> {
        nodes.reverse();
        Selection { nodes }
    }

    pub fn empty() -> Selection<'a> {
        Selection { nodes: vec![] }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn first<T: 'static>(&self) -> Option<&T> {
        for maybe_matching in self.nodes.iter() {
            if let Some(m) = cast::<T>(*maybe_matching) {
                return Some(m);
            }
        }
        None
    }

    pub fn span(&self) -> Option<Span> {
        self.nodes.first()?.span()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

impl fmt::Debug for Selection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Selection(\n")?;
        for n in self.nodes.iter() {
            write!(f, "{} - {:?}\n", n.id().unwrap_or(Id::NULL), n)?;
        }
        write!(f, ")")?;
        Ok(())
    }
}
