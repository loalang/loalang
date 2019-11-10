use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct WrongNumberOfTypeArguments;

impl WrongNumberOfTypeArguments {
    fn check_reference(
        reference: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        let (name, _) = analysis.navigator.symbol_of(&reference)?;
        let args = analysis
            .navigator
            .type_arguments_of_reference_type_expression(reference)
            .len();
        let declaration = analysis
            .navigator
            .find_declaration(reference, DeclarationKind::Type)?;
        let params = analysis
            .navigator
            .type_parameters_of_type_declaration(&declaration)
            .len();

        if args != params {
            diagnostics.push(Diagnostic::WrongNumberOfTypeArguments(
                reference.span.clone(),
                name,
                params,
                args,
            ));
        }

        None
    }
}

impl Checker for WrongNumberOfTypeArguments {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for reference in analysis.navigator.all_reference_type_expressions() {
            Self::check_reference(&reference, analysis, diagnostics).unwrap_or(());
        }
    }
}
