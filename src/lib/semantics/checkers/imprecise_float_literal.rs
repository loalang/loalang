use crate::semantics::*;
use crate::syntax::*;
use crate::BitSize;
use crate::*;
use fraction::ToPrimitive;

pub struct ImpreciseFloatLiteral;

impl ImpreciseFloatLiteral {
    fn check_literal(
        literal: Node,
        type_: Type,
        class: Id,
        analysis: &mut Analysis,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<()> {
        let class = analysis.navigator.find_node(class)?;
        let (name, _, _) = analysis.navigator.qualified_name_of(&class)?;

        match (literal.kind, name.as_str()) {
            (FloatExpression(_, fraction), "Loa/Float32") => {
                Self::assert_fraction_precision(
                    literal.span,
                    type_,
                    fraction,
                    BitSize::Size32,
                    diagnostics,
                );
            }
            (FloatExpression(_, fraction), "Loa/Float64") => {
                Self::assert_fraction_precision(
                    literal.span,
                    type_,
                    fraction,
                    BitSize::Size64,
                    diagnostics,
                );
            }
            _ => {}
        }

        None
    }

    fn assert_fraction_precision(
        span: Span,
        type_: Type,
        fraction: BigFraction,
        size: BitSize,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let big_formatted = format!("{:.1$}", fraction, 310);
        let cast_formatted = match size {
            BitSize::Size32 => fraction.to_f32().unwrap().to_string(),
            BitSize::Size64 => fraction.to_f64().unwrap().to_string(),
            _ => return,
        };

        if big_formatted != cast_formatted {
            diagnostics.push(Diagnostic::TooPreciseFloat(span, type_, fraction))
        }
    }
}

impl Checker for ImpreciseFloatLiteral {
    fn check(&self, analysis: &mut Analysis, diagnostics: &mut Vec<Diagnostic>) {
        for literal in analysis.navigator.all_number_literals() {
            let type_ = analysis.types.get_type_of_expression(&literal);

            match type_ {
                Type::UnresolvedInteger(_, _) => continue,
                Type::UnresolvedFloat(_, _) => continue,
                Type::Class(_, class, _) => {
                    Self::check_literal(literal, type_, class, analysis, diagnostics);
                    continue;
                }
                _ => {}
            }

            diagnostics.push(Diagnostic::InvalidLiteralType(literal.span, type_));
        }
    }
}
