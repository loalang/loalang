use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct VariableInitialization;

impl VariableInitialization {
    fn check_class(
        &self,
        class: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some((class_name, _)) = analysis.navigator.symbol_of(class) {
            let variables = analysis.navigator.variables_of_class(class);
            let variable_names = variables
                .iter()
                .filter_map(|v| analysis.navigator.symbol_of(v))
                .map(|(name, _)| name)
                .collect::<Vec<_>>();
            for initializer in analysis.navigator.initializers_of(class) {
                self.check_initializer(
                    &class_name,
                    &initializer,
                    &variable_names,
                    analysis,
                    diagnostics,
                )
                .unwrap_or(());
            }
        }
    }

    fn check_initializer(
        &self,
        class_name: &String,
        initializer: &Node,
        variable_names: &Vec<String>,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        if let Initializer {
            ref keyword_pairs,
            message_pattern,
            ..
        } = initializer.kind
        {
            let mut uninitialized_names = variable_names.iter().cloned().collect::<HashSet<_>>();
            let mut extraneous_names = vec![];

            analysis
                .navigator
                .keyword_pairs(initializer, keyword_pairs)
                .into_iter()
                .filter_map(|(keyword, _)| analysis.navigator.symbol_of(&keyword))
                .for_each(|(initialized_name, symbol)| {
                    if variable_names.contains(&initialized_name) {
                        uninitialized_names.remove(&initialized_name);
                    } else {
                        extraneous_names.push((initialized_name, symbol));
                    }
                });

            if !uninitialized_names.is_empty() {
                let message_pattern = analysis
                    .navigator
                    .find_child(initializer, message_pattern)?;
                let selector = analysis
                    .navigator
                    .message_pattern_selector(&message_pattern)?;

                let mut uninitialized_names = uninitialized_names.into_iter().collect::<Vec<_>>();

                uninitialized_names.sort();

                diagnostics.push(Diagnostic::IncompleteInitializer(
                    message_pattern.span,
                    selector,
                    uninitialized_names,
                ));
            }

            for (extraneous_name, symbol) in extraneous_names {
                diagnostics.push(Diagnostic::UndefinedInitializedVariable(
                    symbol.span,
                    extraneous_name,
                    class_name.clone(),
                ));
            }
        }
        None
    }
}

impl Checker for VariableInitialization {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for class in analysis.navigator.all_classes() {
            self.check_class(&class, analysis, diagnostics);
        }
    }
}
