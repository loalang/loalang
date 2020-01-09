use crate::semantics::*;
use crate::syntax::TokenKind::PrivateKeyword;
use crate::syntax::*;
use crate::*;

pub struct PrivateMethods;

impl PrivateMethods {
    fn check_method(
        &self,
        method: &Node,
        message: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        if analysis.navigator.visibility_of_method(method)?.kind != PrivateKeyword {
            return None;
        }

        let method_class = analysis.navigator.closest_class_upwards(method)?;
        let message_class = analysis.navigator.closest_class_upwards(message);

        if message_class.is_none() || method_class.id != message_class?.id {
            diagnostics.push(Diagnostic::InvalidAccessToPrivateMethod(
                message.span.clone(),
                analysis.navigator.qualified_name_of(&method_class)?.0,
                analysis.navigator.method_selector(method)?,
            ));
        }
        None
    }

    fn check_message_send(
        &self,
        message_send: &Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        if let MessageSendExpression { message, .. } = message_send.kind {
            let message = analysis.navigator.find_child(&message_send, message)?;
            let method = analysis
                .navigator
                .method_from_message(&message, &analysis.types)?;

            self.check_method(&method, &message, analysis, diagnostics)?;
        }
        None
    }
}

impl Checker for PrivateMethods {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for message_send in analysis.navigator.all_message_sends() {
            self.check_message_send(&message_send, analysis, diagnostics)
                .unwrap_or(());
        }
    }
}
