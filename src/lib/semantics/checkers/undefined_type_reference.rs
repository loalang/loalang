use crate::semantics::*;
use crate::syntax::DeclarationKind;
use crate::*;

pub struct UndefinedTypeReference;

impl Checker for UndefinedTypeReference {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for reference in analysis
            .navigator
            .all_reference_symbols(DeclarationKind::Type)
        {
            if analysis
                .navigator
                .find_usage(&reference, DeclarationKind::Type, &analysis.types)
                .is_none()
            {
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
