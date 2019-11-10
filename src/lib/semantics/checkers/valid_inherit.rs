use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct InvalidInherit;

impl InvalidInherit {
    fn check_is_directive(
        &self,
        directive: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        let class = analysis.navigator.closest_class_upwards(directive)?;
        if let IsDirective {
            type_expression, ..
        } = directive.kind
        {
            let type_expression = analysis.navigator.find_child(directive, type_expression)?;
            let super_type = analysis.types.get_type_of_type_expression(&type_expression);
            let sub_type = analysis.types.get_type_of_declaration(&class);

            self.check_inherit(
                type_expression.span,
                super_type,
                sub_type,
                analysis,
                diagnostics,
            )?;
        }
        None
    }

    fn check_inherit(
        &self,
        span: Span,
        super_type: Type,
        sub_type: Type,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        let mut violations = vec![];

        'super_behaviours: for super_behaviour in analysis.types.get_behaviours(&super_type) {
            let super_selector = super_behaviour.selector();
            'sub_behaviours: for sub_behaviour in analysis.types.get_behaviours(&sub_type) {
                let sub_selector = sub_behaviour.selector();
                if super_selector == sub_selector {
                    let super_behaviour_type =
                        analysis.types.get_type_of_behaviour(&super_behaviour);
                    let sub_behaviour_type = analysis.types.get_type_of_behaviour(&sub_behaviour);

                    let assignment = check_assignment(
                        super_behaviour_type.clone(),
                        sub_behaviour_type.clone(),
                        &analysis.navigator,
                        &analysis.types,
                        false,
                    );
                    if assignment.is_invalid() {
                        violations.push(InheritanceViolation::OverrideNotSound(
                            super_behaviour.clone(),
                            assignment,
                        ));
                    }

                    continue 'super_behaviours;
                }
            }
            violations.push(InheritanceViolation::BehaviourNotImplemented(
                super_behaviour,
            ));
        }

        if violations.len() > 0 {
            diagnostics.push(Diagnostic::InvalidInherit {
                span,
                super_type,
                sub_type,
                violations,
            });
        }

        None
    }
}

impl Checker for InvalidInherit {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for is_directive in analysis.navigator.all_is_directives() {
            self.check_is_directive(&is_directive, analysis, diagnostics)
                .unwrap_or(());
        }
    }
}
