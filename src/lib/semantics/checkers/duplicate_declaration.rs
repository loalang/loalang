use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct DuplicateDeclaration;

impl DuplicateDeclaration {
    fn check_kind(
        kind: DeclarationKind,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
        reported_symbols: &mut Vec<Id>,
    ) {
        let navigator = analysis.navigator();
        for scope_root in navigator.all_scope_roots() {
            let mut declarations_by_name = HashMap::new();

            for declaration in navigator.all_declarations_in_scope(&scope_root, kind) {
                if let Some((name, symbol)) = navigator.symbol_of(&declaration) {
                    if !declarations_by_name.contains_key(&name) {
                        declarations_by_name.insert(name.clone(), vec![]);
                    }

                    declarations_by_name.get_mut(&name).unwrap().push(symbol);
                }
            }

            for (name, declarations) in declarations_by_name {
                let count = declarations.len();

                if count < 2 {
                    continue;
                }

                for symbol in declarations {
                    if reported_symbols.contains(&symbol.id) {
                        continue;
                    }

                    diagnostics.push(Diagnostic::DuplicatedDeclaration(
                        symbol.span,
                        name.clone(),
                        count,
                    ));

                    reported_symbols.push(symbol.id);
                }
            }
        }
    }
}

impl Checker for DuplicateDeclaration {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        let mut reported_symbols = vec![];
        Self::check_kind(DeclarationKind::Type, analysis, diagnostics, &mut reported_symbols);
        Self::check_kind(DeclarationKind::Value, analysis, diagnostics, &mut reported_symbols);
    }
}
