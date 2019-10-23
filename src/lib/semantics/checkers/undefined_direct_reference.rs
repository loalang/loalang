use crate::semantics::*;
use crate::*;

pub struct UndefinedDirectReference;

impl Checker for UndefinedDirectReference {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for reference in analysis.all_reference_symbols() {
            if analysis.usage(&reference).is_none() {
                if let syntax::Symbol(t) = reference.kind {
                    diagnostics.push(Diagnostic::UndefinedTypeReference(
                        t.span.clone(),
                        t.lexeme(),
                    ))
                }
            }
        }
    }
}
