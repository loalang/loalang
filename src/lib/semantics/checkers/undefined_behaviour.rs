use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct UndefinedBehaviour;

impl UndefinedBehaviour {
    fn check_message_send(
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
            let message = analysis.navigator.find_child(message_send, message)?;
            let selector = analysis.navigator.message_selector(&message)?;

            let receiver_type = analysis.types.get_type_of_expression(&expression);
            if receiver_type.is_unknown() {
                return None;
            }

            for behaviour in analysis.types.get_behaviours(&receiver_type) {
                if behaviour.selector() == selector {
                    return None;
                }
            }

            diagnostics.push(Diagnostic::UndefinedBehaviour(
                message.span,
                receiver_type,
                selector,
            ))
        }

        None
    }
}

impl Checker for UndefinedBehaviour {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for message_send in analysis.navigator.all_message_sends() {
            Self::check_message_send(&message_send, analysis, diagnostics).unwrap_or(());
        }
    }
}
