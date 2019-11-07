use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub struct OutOfBoundsInteger;

impl OutOfBoundsInteger {
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
            (IntegerExpression(_, int), "Loa/UInt8") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    0.into(),
                    "0",
                    std::u8::MAX.into(),
                    "2ˆ8-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/UInt16") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    0.into(),
                    "0",
                    std::u16::MAX.into(),
                    "2ˆ16-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/UInt32") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    0.into(),
                    "0",
                    std::u32::MAX.into(),
                    "2ˆ32-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/UInt64") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    0.into(),
                    "0",
                    std::u64::MAX.into(),
                    "2ˆ64-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/UInt128") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    0.into(),
                    "0",
                    std::u128::MAX.into(),
                    "2ˆ128-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/Int8") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    std::i8::MIN.into(),
                    "-(2ˆ7-1)",
                    std::i8::MAX.into(),
                    "2ˆ7-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/Int16") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    std::i16::MIN.into(),
                    "-(2ˆ15-1)",
                    std::i16::MAX.into(),
                    "2ˆ15-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/Int32") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    std::i32::MIN.into(),
                    "-(2ˆ31-1)",
                    std::i32::MAX.into(),
                    "2ˆ31-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/Int64") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    std::i64::MIN.into(),
                    "-(2ˆ63-1)",
                    std::i64::MAX.into(),
                    "2ˆ63-1",
                    diagnostics,
                );
            }
            (IntegerExpression(_, int), "Loa/Int128") => {
                Self::assert_int_in_bound(
                    literal.span,
                    type_,
                    int,
                    std::i128::MIN.into(),
                    "-(2ˆ127-1)",
                    std::i128::MAX.into(),
                    "2ˆ127-1",
                    diagnostics,
                );
            }
            _ => {}
        }

        None
    }

    fn assert_int_in_bound(
        span: Span,
        type_: Type,
        int: BigInt,
        min: BigInt,
        min_str: &str,
        max: BigInt,
        max_str: &str,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if int < min {
            diagnostics.push(Diagnostic::OutOfBounds(
                span,
                type_,
                format!("less than {}", min_str),
            ));
        } else if int > max {
            diagnostics.push(Diagnostic::OutOfBounds(
                span,
                type_,
                format!("greater than {}", max_str),
            ));
        }
    }
}

impl Checker for OutOfBoundsInteger {
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
