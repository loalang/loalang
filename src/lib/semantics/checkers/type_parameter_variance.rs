use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct TypeParameterVariance;

#[derive(PartialEq)]
enum SignaturePosition {
    In,
    Out,
}

impl TypeParameterVariance {
    fn type_parameter_is_used_in_type(&self, type_: Type, type_parameter: &Node) -> bool {
        match type_ {
            Type::Self_(_)
            | Type::Unknown
            | Type::Symbol(_)
            | Type::UnresolvedInteger(_, _)
            | Type::UnresolvedFloat(_, _) => false,

            Type::Class(_, _, args) => {
                for arg in args {
                    if self.type_parameter_is_used_in_type(arg, type_parameter) {
                        return true;
                    }
                }
                false
            }

            Type::Behaviour(box Behaviour {
                message,
                return_type,
                ..
            }) => {
                if self.type_parameter_is_used_in_type(return_type, type_parameter) {
                    return true;
                }
                self.type_parameter_is_used_in_message(message, type_parameter)
            }

            Type::Parameter(_, id, args) => {
                if id == type_parameter.id {
                    return true;
                }
                for arg in args {
                    if self.type_parameter_is_used_in_type(arg, type_parameter) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn type_parameter_is_used_in_message(
        &self,
        message: BehaviourMessage,
        type_parameter: &Node,
    ) -> bool {
        match message {
            BehaviourMessage::Unary(_) => false,
            BehaviourMessage::Binary(_, t) => {
                self.type_parameter_is_used_in_type(t, type_parameter)
            }
            BehaviourMessage::Keyword(kws) => {
                for (_, t) in kws {
                    if self.type_parameter_is_used_in_type(t, type_parameter) {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn check_behaviour_usage_of_type_parameter(
        &self,
        behaviour: Behaviour,
        type_parameter: &Node,
        expected_position: &SignaturePosition,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        match expected_position {
            SignaturePosition::In => {
                if self.type_parameter_is_used_in_type(behaviour.return_type, type_parameter) {
                    let method = analysis.navigator.find_node(behaviour.method_id)?;
                    diagnostics.push(Diagnostic::InvalidTypeParameterReferenceVarianceUsage(
                        method.span,
                        analysis.navigator.symbol_of(type_parameter)?.0,
                        "output",
                        "in",
                    ));
                }
            }
            SignaturePosition::Out => {
                if self.type_parameter_is_used_in_message(behaviour.message, type_parameter) {
                    let method = analysis.navigator.find_node(behaviour.method_id)?;
                    diagnostics.push(Diagnostic::InvalidTypeParameterReferenceVarianceUsage(
                        method.span,
                        analysis.navigator.symbol_of(type_parameter)?.0,
                        "input",
                        "out",
                    ));
                }
            }
        }
        None
    }

    fn check_valid_type_parameter(
        &self,
        type_parameter: Node,
        expected_position: SignaturePosition,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        let param_list = analysis.navigator.parent(&type_parameter)?;
        let declaration = analysis.navigator.parent(&param_list)?;

        match declaration.kind {
            Signature { .. } => {
                let method = analysis.navigator.parent(&declaration)?;
                let class = analysis.navigator.closest_class_upwards(&method)?;
                let behaviour = analysis.types.get_behaviour_from_method(
                    analysis.types.get_type_of_declaration(&class),
                    method,
                )?;
                self.check_behaviour_usage_of_type_parameter(
                    behaviour,
                    &type_parameter,
                    &expected_position,
                    analysis,
                    diagnostics,
                )?;
            }
            Class { .. } => {
                let class_type = analysis.types.get_type_of_declaration(&declaration);
                for behaviour in analysis.types.get_behaviours(&class_type) {
                    self.check_behaviour_usage_of_type_parameter(
                        behaviour,
                        &type_parameter,
                        &expected_position,
                        analysis,
                        diagnostics,
                    )
                    .unwrap_or(());
                }
            }
            _ => {}
        }
        None
    }

    fn check_type_parameter(
        &self,
        type_parameter: Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        if let TypeParameter {
            ref variance_keyword,
            ..
        } = type_parameter.kind
        {
            match variance_keyword.clone()?.kind {
                TokenKind::InKeyword => self.check_valid_type_parameter(
                    type_parameter,
                    SignaturePosition::In,
                    analysis,
                    diagnostics,
                ),
                TokenKind::OutKeyword => self.check_valid_type_parameter(
                    type_parameter,
                    SignaturePosition::Out,
                    analysis,
                    diagnostics,
                ),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Checker for TypeParameterVariance {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for type_parameter in analysis.navigator.all_type_parameters() {
            self.check_type_parameter(type_parameter, analysis, diagnostics)
                .unwrap_or(());
        }
    }
}
