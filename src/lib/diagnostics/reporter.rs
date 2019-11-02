use crate::semantics::Navigator;
use crate::*;

pub trait Reporter {
    fn report(diagnostics: Vec<Diagnostic>, navigator: &Navigator);
}

pub struct BasicReporter;

impl Reporter for BasicReporter {
    fn report(diagnostics: Vec<Diagnostic>, _navigator: &Navigator) {
        for diagnostic in diagnostics {
            println!("{}: {:?}", diagnostic.span(), diagnostic);
        }
    }
}
