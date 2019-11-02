use crate::semantics::Navigator;
use crate::*;

pub trait Reporter {
    fn report(diagnostic: Diagnostic, navigator: &Navigator);
}

pub struct BasicReporter;

impl Reporter for BasicReporter {
    fn report(diagnostic: Diagnostic, _navigator: &Navigator) {
        println!("{}: {:?}", diagnostic.span(), diagnostic);
    }
}
