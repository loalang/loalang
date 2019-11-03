use crate::semantics::*;
use crate::syntax::DeclarationKind;
use crate::*;

pub struct UndefinedReference;

impl Checker for UndefinedReference {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for reference in analysis
            .navigator
            .all_reference_symbols(DeclarationKind::Value)
        {
            if analysis
                .navigator
                .find_usage(&reference, DeclarationKind::Value, &analysis.types)
                .is_none()
            {
                if let syntax::Symbol(t) = reference.kind {
                    diagnostics.push(Diagnostic::UndefinedReference(t.span.clone(), t.lexeme()))
                }
            }
        }
    }
}
