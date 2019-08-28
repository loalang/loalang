use crate::*;

pub trait Reporter {
    fn report(&self, diagnostics: &Vec<Diagnostic>);
}

pub struct BasicReporter;

impl Reporter for BasicReporter {
    fn report(&self, diagnostics: &Vec<Diagnostic>) {
        for diagnostic in diagnostics.iter() {
            match diagnostic.span() {
                Some(span) => println!("{}: {:?}", span, diagnostic),
                None => println!("{:?}", diagnostic),
            }
        }
    }
}
