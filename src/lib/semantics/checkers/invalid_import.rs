use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct InvalidImport;

impl Checker for InvalidImport {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for import in analysis.navigator.all_imports() {
            if let ImportDirective {
                qualified_symbol, ..
            } = import.kind
            {
                if let Some(qualified_symbol) =
                    analysis.navigator.find_child(&import, qualified_symbol)
                {
                    let name = analysis
                        .navigator
                        .qualified_symbol_to_string(&qualified_symbol);
                    match analysis.navigator.find_declaration_from_import(&import) {
                        None => diagnostics
                            .push(Diagnostic::UndefinedImport(qualified_symbol.span, name)),
                        Some(declaration) => {
                            if !analysis.navigator.declaration_is_exported(&declaration) {
                                diagnostics.push(Diagnostic::UnexportedImport(
                                    qualified_symbol.span,
                                    name,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}
