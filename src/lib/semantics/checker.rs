use crate::semantics::*;
use crate::*;

pub trait Checker {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>);
}
