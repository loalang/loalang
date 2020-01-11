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
    fn position_in_signature(
        &self,
        node: &Node,
        analysis: &mut Analysis,
    ) -> Option<SignaturePosition> {
        let mut parent = analysis.navigator.parent(node)?;
        loop {
            match parent.kind {
                NodeKind::ReturnType { .. } => return Some(SignaturePosition::Out),
                NodeKind::Signature { .. } => return Some(SignaturePosition::In),
                _ => {
                    parent = analysis.navigator.parent(&parent)?;
                }
            }
        }
    }

    fn is_in_input_position(&self, node: &Node, analysis: &mut Analysis) -> Option<bool> {
        self.position_in_signature(node, analysis)
            .map(|p| p == SignaturePosition::In)
    }

    fn is_in_output_position(&self, node: &Node, analysis: &mut Analysis) -> Option<bool> {
        self.position_in_signature(node, analysis)
            .map(|p| p == SignaturePosition::Out)
    }

    fn check_contravariant_type_parameter(
        &self,
        type_parameter: Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        for reference in analysis
            .navigator
            .find_references(&type_parameter, DeclarationKind::Type)
        {
            if let Some(true) = self.is_in_output_position(&reference, analysis) {
                let name = analysis.navigator.symbol_of(&reference)?.0;
                diagnostics.push(Diagnostic::InvalidTypeParameterReferenceVarianceUsage(
                    reference.span,
                    name,
                    "output",
                    "in",
                ));
            }
        }
        None
    }

    fn check_covariant_type_parameter(
        &self,
        type_parameter: Node,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        for reference in analysis
            .navigator
            .find_references(&type_parameter, DeclarationKind::Type)
        {
            if let Some(true) = self.is_in_input_position(&reference, analysis) {
                let name = analysis.navigator.symbol_of(&reference)?.0;
                diagnostics.push(Diagnostic::InvalidTypeParameterReferenceVarianceUsage(
                    reference.span,
                    name,
                    "input",
                    "out",
                ));
            }
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
                TokenKind::InKeyword => {
                    self.check_contravariant_type_parameter(type_parameter, analysis, diagnostics)
                }
                TokenKind::OutKeyword => {
                    self.check_covariant_type_parameter(type_parameter, analysis, diagnostics)
                }
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
