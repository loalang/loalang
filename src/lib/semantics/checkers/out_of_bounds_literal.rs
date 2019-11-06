use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct OutOfBoundsLiteral;

impl Checker for OutOfBoundsLiteral {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for literal in analysis.navigator.all_number_literals() {
            let type_ = analysis.types.get_type_of_expression(&literal);

            info!("Make sure that {:?} fits in {}", literal.kind, type_)
        }
    }
}
