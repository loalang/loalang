use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct TypeAssignment;

impl TypeAssignment {
    fn diagnose_assignment(
        &self,
        span: Span,
        assignee: Type,
        assigned: Type,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let assignability = check_assignment(
            assignee,
            assigned,
            &analysis.navigator,
            &analysis.types,
            false,
        );

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
                let behaviour =
                    behaviour.with_applied_message(&message, &analysis.navigator, &analysis.types);
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

    fn check_let_binding(
        &self,
        let_binding: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        if let LetBinding {
            expression,
            type_expression,
            ..
        } = let_binding.kind
        {
            let expression = analysis.navigator.find_child(let_binding, expression)?;
            let type_expression = analysis
                .navigator
                .find_child(let_binding, type_expression)?;

            let assigned = analysis.types.get_type_of_expression(&expression);
            let assignee = analysis.types.get_type_of_type_expression(&type_expression);

            self.diagnose_assignment(expression.span, assignee, assigned, analysis, diagnostics);
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
            if n.is_let_binding() {
                self.check_let_binding(n, analysis, diagnostics)
                    .unwrap_or(());
            }
            true
        })
    }
}
