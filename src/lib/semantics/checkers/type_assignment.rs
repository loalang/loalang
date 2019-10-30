use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct TypeAssignment;

impl TypeAssignment {
    fn check_assignment(
        &self,
        assignee: Type,
        assigned: Type,
        analysis: &mut Analysis,
        invariant: bool,
    ) -> TypeAssignability {
        match (&assignee, &assigned) {
            (
                Type::Class(_, assignee_class, assignee_args),
                Type::Class(_, assigned_class, assigned_args),
            ) => {
                let assignee_class = analysis.navigator.find_node(*assignee_class)?;
                let assigned_class = analysis.navigator.find_node(*assigned_class)?;

                if assignee_class.id != assigned_class.id {
                    if !invariant {
                        for super_type in analysis.navigator.super_type_expressions(&assigned_class)
                        {
                            let super_type =
                                analysis.types.get_type_of_type_expression(&super_type);

                            let assigned_super_type_assignability = self.check_assignment(
                                assignee.clone(),
                                super_type,
                                analysis,
                                false,
                            );

                            if assigned_super_type_assignability.is_valid() {
                                return assigned_super_type_assignability;
                            }
                        }
                    }

                    return TypeAssignability::Invalid {
                        assigned,
                        assignee,
                        invariant,
                        because: vec![],
                    };
                }

                if let Class {
                    type_parameter_list,
                    ..
                } = assignee_class.kind
                {
                    let type_parameter_list = analysis
                        .navigator
                        .find_child(&assignee_class, type_parameter_list)?;
                    if let TypeParameterList {
                        ref type_parameters,
                        ..
                    } = type_parameter_list.kind
                    {
                        let mut issues = vec![];

                        for (i, parameter) in type_parameters.iter().enumerate() {
                            let assignee_arg = if assignee_args.len() > i {
                                assignee_args[i].clone()
                            } else {
                                Type::Unknown
                            };

                            let assigned_arg = if assigned_args.len() > i {
                                assigned_args[i].clone()
                            } else {
                                Type::Unknown
                            };

                            let parameter = analysis
                                .navigator
                                .find_child(&type_parameter_list, *parameter)?;
                            if let TypeParameter {
                                variance_keyword, ..
                            } = parameter.kind
                            {
                                let arg_assignability = match variance_keyword.map(|t| t.kind) {
                                    None | Some(TokenKind::InoutKeyword) => self.check_assignment(
                                        assignee_arg,
                                        assigned_arg,
                                        analysis,
                                        true,
                                    ),
                                    Some(TokenKind::OutKeyword) => self.check_assignment(
                                        assignee_arg,
                                        assigned_arg,
                                        analysis,
                                        false,
                                    ),
                                    Some(TokenKind::InKeyword) => self.check_assignment(
                                        assigned_arg,
                                        assignee_arg,
                                        analysis,
                                        false,
                                    ),
                                    _ => TypeAssignability::Valid,
                                };

                                if arg_assignability.is_invalid() {
                                    issues.push(arg_assignability);
                                }
                            }
                        }

                        if issues.len() > 0 {
                            return TypeAssignability::Invalid {
                                assigned,
                                assignee,
                                invariant,
                                because: issues,
                            };
                        }
                    }
                }
            }
            _ => {}
        }

        TypeAssignability::Valid
    }

    fn diagnose_assignment(
        &self,
        span: Span,
        assignee: Type,
        assigned: Type,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let assignability = self.check_assignment(assignee, assigned, analysis, false);

        if assignability.is_invalid() {
            diagnostics.push(Diagnostic::UnassignableType {
                span,
                assignability,
            });
        }
    }

    fn check_method(
        &self,
        method: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        if let Method {
            method_body,
            signature,
            ..
        } = method.kind
        {
            let signature = analysis.navigator.find_child(method, signature)?;
            if let Signature { return_type, .. } = signature.kind {
                let return_type = analysis.navigator.find_child(&signature, return_type)?;
                if let ReturnType {
                    type_expression, ..
                } = return_type.kind
                {
                    let type_expression = analysis
                        .navigator
                        .find_child(&return_type, type_expression)?;
                    let assignee = analysis.types.get_type_of_type_expression(&type_expression);

                    let method_body = analysis.navigator.find_child(method, method_body)?;
                    if let MethodBody { expression, .. } = method_body.kind {
                        let expression = analysis.navigator.find_child(&method_body, expression)?;
                        let assigned = analysis.types.get_type_of_expression(&expression);

                        self.diagnose_assignment(
                            expression.span,
                            assignee,
                            assigned,
                            analysis,
                            diagnostics,
                        );
                    }
                }
            }
        }
        Some(())
    }

    fn check_message_send(
        &self,
        message_send: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        if let MessageSendExpression {
            expression,
            message,
            ..
        } = message_send.kind
        {
            let expression = analysis.navigator.find_child(message_send, expression)?;
            let receiver_type = analysis.types.get_type_of_expression(&expression);

            let message = analysis.navigator.find_child(message_send, message)?;
            let selector = analysis.navigator.message_selector(&message)?;

            let behaviours = analysis.types.get_behaviours(&receiver_type);

            for behaviour in behaviours {
                if behaviour.selector() == selector {
                    match (behaviour.message, &message.kind) {
                        (BehaviourMessage::Unary(_), UnaryMessage { .. }) => {}
                        (
                            BehaviourMessage::Binary(_, ref parameter_type),
                            BinaryMessage { expression, .. },
                        ) => {
                            let argument = analysis.navigator.find_child(&message, *expression)?;
                            let argument_type = analysis.types.get_type_of_expression(&argument);

                            self.diagnose_assignment(
                                argument.span,
                                parameter_type.clone(),
                                argument_type,
                                analysis,
                                diagnostics,
                            );
                        }
                        (
                            BehaviourMessage::Keyword(ref kws),
                            KeywordMessage { ref keyword_pairs },
                        ) => {
                            for (i, (_, parameter_type)) in kws.iter().enumerate() {
                                let keyword_pair =
                                    analysis.navigator.find_child(&message, keyword_pairs[i])?;
                                if let KeywordPair { value, .. } = keyword_pair.kind {
                                    let argument =
                                        analysis.navigator.find_child(&message, value)?;
                                    let argument_type =
                                        analysis.types.get_type_of_expression(&argument);

                                    self.diagnose_assignment(
                                        argument.span,
                                        parameter_type.clone(),
                                        argument_type,
                                        analysis,
                                        diagnostics,
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        None
    }
}

impl Checker for TypeAssignment {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        analysis.navigator.clone().traverse_all(&mut |n| {
            if n.is_method() {
                self.check_method(n, analysis, diagnostics).unwrap_or(());
            }
            if n.is_message_send() {
                self.check_message_send(n, analysis, diagnostics)
                    .unwrap_or(());
            }
            true
        })
    }
}
