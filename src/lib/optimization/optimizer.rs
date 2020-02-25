use crate::assembly::*;
use crate::vm::NativeMethod;

pub struct Optimizer {
    sections: Vec<(String, Section)>,
    marked_sections: Vec<String>,

    true_class_label: Option<String>,
    false_class_label: Option<String>,
    string_class_label: Option<String>,
    character_class_label: Option<String>,
    symbol_class_label: Option<String>,
    u8_class_label: Option<String>,
    u16_class_label: Option<String>,
    u32_class_label: Option<String>,
    u64_class_label: Option<String>,
    u128_class_label: Option<String>,
    ubig_class_label: Option<String>,
    i8_class_label: Option<String>,
    i16_class_label: Option<String>,
    i32_class_label: Option<String>,
    i64_class_label: Option<String>,
    i128_class_label: Option<String>,
    ibig_class_label: Option<String>,
    f32_class_label: Option<String>,
    f64_class_label: Option<String>,
    fbig_class_label: Option<String>,
}

impl Optimizer {
    pub fn new(assembly: Assembly) -> Optimizer {
        let mut label_gen = 0;
        Optimizer {
            sections: assembly
                .into_iter()
                .map(|s| {
                    (
                        s.label.clone().unwrap_or_else(|| {
                            label_gen += 1;
                            format!("$generatedLabel{}", label_gen)
                        }),
                        s,
                    )
                })
                .collect(),
            marked_sections: vec![],

            true_class_label: None,
            false_class_label: None,
            string_class_label: None,
            character_class_label: None,
            symbol_class_label: None,
            u8_class_label: None,
            u16_class_label: None,
            u32_class_label: None,
            u64_class_label: None,
            u128_class_label: None,
            ubig_class_label: None,
            i8_class_label: None,
            i16_class_label: None,
            i32_class_label: None,
            i64_class_label: None,
            i128_class_label: None,
            ibig_class_label: None,
            f32_class_label: None,
            f64_class_label: None,
            fbig_class_label: None,
        }
    }

    pub fn optimize(mut self) -> Assembly {
        self.collect_const_class_marks();
        self.mark_from_beginning();
        self.mark_from_marks();

        let mut optimized = Assembly::new();

        for (label, mut section) in self.sections {
            if self.marked_sections.contains(&label) {
                Self::optimize_section(&mut section, &self.marked_sections);
                if section.is_empty() && section.label.is_some() {
                    section.add_instruction(InstructionKind::Noop);
                }
                if !section.is_empty() {
                    optimized.add_section(section);
                }
            }
        }

        optimized
    }

    fn collect_const_class_marks(&mut self) {
        for (label, section) in self.sections.iter() {
            for instruction in section.instructions.iter() {
                match instruction.kind {
                    InstructionKind::MarkClassTrue(_) => {
                        self.true_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassFalse(_) => {
                        self.false_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassString(_) => {
                        self.string_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassCharacter(_) => {
                        self.character_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassSymbol(_) => {
                        self.symbol_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassU8(_) => self.u8_class_label = Some(label.clone()),
                    InstructionKind::MarkClassU16(_) => self.u16_class_label = Some(label.clone()),
                    InstructionKind::MarkClassU32(_) => self.u32_class_label = Some(label.clone()),
                    InstructionKind::MarkClassU64(_) => self.u64_class_label = Some(label.clone()),
                    InstructionKind::MarkClassU128(_) => {
                        self.u128_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassUBig(_) => {
                        self.ubig_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassI8(_) => self.i8_class_label = Some(label.clone()),
                    InstructionKind::MarkClassI16(_) => self.i16_class_label = Some(label.clone()),
                    InstructionKind::MarkClassI32(_) => self.i32_class_label = Some(label.clone()),
                    InstructionKind::MarkClassI64(_) => self.i64_class_label = Some(label.clone()),
                    InstructionKind::MarkClassI128(_) => {
                        self.i128_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassIBig(_) => {
                        self.ibig_class_label = Some(label.clone())
                    }
                    InstructionKind::MarkClassF32(_) => self.f32_class_label = Some(label.clone()),
                    InstructionKind::MarkClassF64(_) => self.f64_class_label = Some(label.clone()),
                    InstructionKind::MarkClassFBig(_) => {
                        self.fbig_class_label = Some(label.clone())
                    }
                    _ => {}
                }
            }
        }
    }

    fn optimize_section(section: &mut Section, marks: &Vec<String>) {
        section.instructions.retain(|i| match i.kind {
            InstructionKind::Noop => false,
            InstructionKind::UseMethod(ref l) | InstructionKind::DeclareMethod(_, ref l) => {
                marks.contains(l)
            }
            _ => true,
        })
    }

    fn mark_from_beginning(&mut self) {
        'sections: for (label, section) in self.sections.iter() {
            let mut marked = false;
            for instruction in section.instructions.iter() {
                if marked {
                    if let InstructionKind::Halt = instruction.kind {
                        break 'sections;
                    } else {
                        continue;
                    }
                }

                match instruction.kind {
                    InstructionKind::Halt => {
                        self.marked_sections.push(label.clone());
                        break 'sections;
                    }

                    // Instructions that, by themselves, don't
                    // warrant a marked section.
                    InstructionKind::Noop
                    | InstructionKind::DeclareClass(_)
                    | InstructionKind::DeclareMethod(_, _)
                    | InstructionKind::DeclareVariable(_, _, _, _)
                    | InstructionKind::UseMethod(_)
                    | InstructionKind::OverrideMethod(_, _)
                    | InstructionKind::UseVariable(_)
                    | InstructionKind::MarkClassTrue(_)
                    | InstructionKind::MarkClassFalse(_)
                    | InstructionKind::MarkClassString(_)
                    | InstructionKind::MarkClassCharacter(_)
                    | InstructionKind::MarkClassSymbol(_)
                    | InstructionKind::MarkClassU8(_)
                    | InstructionKind::MarkClassU16(_)
                    | InstructionKind::MarkClassU32(_)
                    | InstructionKind::MarkClassU64(_)
                    | InstructionKind::MarkClassU128(_)
                    | InstructionKind::MarkClassUBig(_)
                    | InstructionKind::MarkClassI8(_)
                    | InstructionKind::MarkClassI16(_)
                    | InstructionKind::MarkClassI32(_)
                    | InstructionKind::MarkClassI64(_)
                    | InstructionKind::MarkClassI128(_)
                    | InstructionKind::MarkClassIBig(_)
                    | InstructionKind::MarkClassF32(_)
                    | InstructionKind::MarkClassF64(_)
                    | InstructionKind::MarkClassFBig(_) => {}

                    _ => {
                        self.marked_sections.push(label.clone());
                        marked = true;
                    }
                }
            }
        }
    }

    fn mark_from_marks(&mut self) {
        let mut queued_marks = self.marked_sections.clone();
        macro_rules! mark {
            ($label:expr) => {{
                let label = $label;
                if !self.marked_sections.contains(label) {
                    self.marked_sections.push(label.clone());
                    queued_marks.push(label.clone());
                }
            }};
            (?$label:expr) => {{
                if let Some(ref l) = $label {
                    mark!(l);
                }
            }};
        }
        while !queued_marks.is_empty() {
            let mark = queued_marks.remove(0);
            if let Some((_, section)) = self.sections.iter().filter(|(l, _)| l == &mark).next() {
                for instruction in section.instructions.iter() {
                    match instruction.kind {
                        InstructionKind::LoadObject(ref label)
                            | InstructionKind::LoadLazy(_, ref label)
                            => {
                            mark!(label);
                        }
                         InstructionKind::DeclareVariable(_, ref vl, ref gl, ref sl) => {
                            mark!(vl);
                            mark!(gl);
                            mark!(sl);
                         }
                        InstructionKind::CallMethod(ref label, _, _, _) => {
                            // Mark method implementation as used
                            mark!(label);

                            // Mark method declaration
                            if let Some(class_name) = label.split("#").next() {
                                mark!(&format!("{}$methods", class_name));
                            }
                        }
                        InstructionKind::OverrideMethod(ref source, ref target) => {
                            // Mark implementations
                            mark!(source);
                            mark!(target);

                            // Mark declarations
                            if let Some(class_name) = source.split("#").next() {
                                mark!(&format!("{}$methods", class_name));
                            }
                            if let Some(class_name) = target.split("#").next() {
                                mark!(&format!("{}$methods", class_name));
                            }
                        }

                        InstructionKind::CallNative(NativeMethod::Object_eq) => {
                            mark!(?self.true_class_label);
                            mark!(?self.false_class_label);
                        }

                        InstructionKind::LoadConstString(_) => mark!(?self.string_class_label),
                        InstructionKind::LoadConstCharacter(_) => {
                            mark!(?self.character_class_label)
                        }
                        InstructionKind::LoadConstSymbol(_) => mark!(?self.symbol_class_label),
                        InstructionKind::LoadConstU8(_) => mark!(?self.u8_class_label),
                        InstructionKind::LoadConstU16(_) => mark!(?self.u16_class_label),
                        InstructionKind::LoadConstU32(_) => mark!(?self.u32_class_label),
                        InstructionKind::LoadConstU64(_) => mark!(?self.u64_class_label),
                        InstructionKind::LoadConstU128(_) => mark!(?self.u128_class_label),
                        InstructionKind::LoadConstUBig(_) => mark!(?self.ubig_class_label),
                        InstructionKind::LoadConstI8(_) => mark!(?self.i8_class_label),
                        InstructionKind::LoadConstI16(_) => mark!(?self.i16_class_label),
                        InstructionKind::LoadConstI32(_) => mark!(?self.i32_class_label),
                        InstructionKind::LoadConstI64(_) => mark!(?self.i64_class_label),
                        InstructionKind::LoadConstI128(_) => mark!(?self.i128_class_label),
                        InstructionKind::LoadConstIBig(_) => mark!(?self.ibig_class_label),
                        InstructionKind::LoadConstF32(_) => mark!(?self.f32_class_label),
                        InstructionKind::LoadConstF64(_) => mark!(?self.f64_class_label),
                        InstructionKind::LoadConstFBig(_) => mark!(?self.fbig_class_label),
                        _ => {}
                    }
                }
            }
        }
    }
}

pub trait Optimizable {
    fn optimize(&mut self);
}

impl Optimizable for Assembly {
    fn optimize(&mut self) {
        let this = std::mem::replace(self, Assembly::new());
        let optimized = Optimizer::new(this).optimize();
        std::mem::replace(self, optimized);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assembly(code: &str) -> Assembly {
        Parser::new().parse(code).unwrap()
    }

    fn assert_optimizes(input: &str, expected: &str) {
        let input = assembly(input);

        let optimizer = Optimizer::new(input);
        let output = optimizer.optimize();

        let expected = assembly(expected);

        assert_eq!(output, expected);
    }

    #[test]
    fn noop_assembly() {
        assert_optimizes(
            r#"
                Noop
            "#,
            r#"
            "#,
        );
    }

    #[test]
    fn single_unused_label() {
        assert_optimizes(
            r#"
                Halt

                @unused
                    Noop
            "#,
            r#"
                Halt
            "#,
        );
    }

    #[test]
    fn only_used_classes_retained() {
        assert_optimizes(
            r#"
                @A
                    DeclareClass "A"

                @B
                    DeclareClass "B"

                LoadObject @A
                Halt
            "#,
            r#"
                @A
                    DeclareClass "A"

                LoadObject @A
                Halt
            "#,
        );
    }
}
