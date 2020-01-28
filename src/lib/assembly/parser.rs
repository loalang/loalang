use crate::assembly::*;
use std::num::ParseIntError;

pub struct Parser;

impl Parser {
    pub fn new() -> Parser {
        Parser
    }

    pub fn parse(&self, code: &str) -> ParseResult<Assembly> {
        let mut assembly = Assembly::new();
        let mut code = String::from(code.trim_start());
        Self::skip_leading_whitespace(&mut code);
        while !code.is_empty() {
            assembly.sections.push(self.parse_section(&mut code)?);
            Self::skip_leading_whitespace(&mut code);
        }
        Ok(assembly)
    }

    fn skip_leading_whitespace(s: &mut String) {
        let trimmed = s.trim_start();
        let trim_start = trimmed.as_ptr() as usize - s.as_ptr() as usize;
        if trim_start > 0 {
            s.drain(..trim_start);
        }
    }

    fn parse_section(&self, code: &mut String) -> ParseResult<Section> {
        let mut section = Section::unnamed();

        if code.starts_with(";") {
            section.leading_comment = Some(self.parse_comment(code)?);
        }

        Self::skip_leading_whitespace(code);
        if code.starts_with("@") {
            section.label = Some(self.parse_label(code)?);
        }

        Self::skip_leading_whitespace(code);
        while !code.is_empty() {
            let mut leading_comment = None;
            if code.starts_with(";") {
                leading_comment = Some(self.parse_comment(code)?);
            }
            if code.starts_with("@") {
                break;
            } else if code.starts_with("Noop") {
                code.drain(.."Noop".len());
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Noop,
                });
            } else if code.starts_with("Halt") {
                code.drain(.."Halt".len());
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Halt,
                });
            } else if code.starts_with("DeclareClass") {
                code.drain(.."DeclareClass".len());
                let name = self.parse_string(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::DeclareClass(name),
                });
            } else if code.starts_with("DeclareMethod") {
                code.drain(.."DeclareMethod".len());
                let name = self.parse_string(code)?;
                let label = self.parse_label(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::DeclareMethod(name, label),
                });
            } else if code.starts_with("LoadObject") {
                code.drain(.."LoadObject".len());
                let label = self.parse_label(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::LoadObject(label),
                });
            } else if code.starts_with("CallMethod") {
                code.drain(.."CallMethod".len());
                let label = self.parse_label(code)?;
                let uri = self.parse_string(code)?;
                let line = self.parse_u64(code)?;
                let character = self.parse_u64(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::CallMethod(label, uri, line, character),
                });
            } else if code.starts_with("LoadLocal") {
                code.drain(.."LoadLocal".len());
                let index = self.parse_u16(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::LoadLocal(index),
                });
            } else if code.starts_with("Return") {
                code.drain(.."Return".len());
                let arity = self.parse_u16(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Return(arity),
                });
            } else {
                return Err(ParseError::ExpectedInstruction(code.clone()));
            }
            Self::skip_leading_whitespace(code);
        }

        Ok(section)
    }

    fn parse_string(&self, code: &mut String) -> ParseResult<String> {
        Self::skip_leading_whitespace(code);
        if !code.starts_with("\"") {
            return Err(ParseError::ExpectedString(code.clone()));
        }
        code.drain(.."\"".len());
        let mut result = String::new();
        while code.len() > 0 && !code.starts_with("\"") {
            result.push(code.remove(0));
        }
        if code.starts_with("\"") {
            code.drain(.."\"".len());
        }
        Ok(result)
    }

    fn parse_comment(&self, code: &mut String) -> ParseResult<String> {
        let mut result = String::new();
        while code.starts_with(";") {
            code.drain(..";".len());
            while code.len() > 0 && !code.starts_with("\n") {
                result.push(code.remove(0));
            }
            Self::skip_leading_whitespace(code);
        }
        Ok(result)
    }

    fn parse_label(&self, code: &mut String) -> ParseResult<Label> {
        Self::skip_leading_whitespace(code);
        if !code.starts_with("@") {
            return Err(ParseError::ExpectedLabel(code.clone()));
        }
        code.drain(.."@".len());
        let mut label = String::new();
        while code.len() > 0 && !(code.as_bytes()[0] as char).is_whitespace() {
            label.push(code.remove(0));
        }
        Ok(label)
    }

    fn parse_u16(&self, code: &mut String) -> ParseResult<u16> {
        Self::skip_leading_whitespace(code);
        let mut number = String::new();
        while code.len() > 0 && (code.as_bytes()[0] as char).is_numeric() {
            number.push(code.remove(0));
        }
        Ok(number.parse()?)
    }

    fn parse_u64(&self, code: &mut String) -> ParseResult<u64> {
        Self::skip_leading_whitespace(code);
        let mut number = String::new();
        while code.len() > 0 && (code.as_bytes()[0] as char).is_numeric() {
            number.push(code.remove(0));
        }
        Ok(number.parse()?)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    ExpectedInstruction(String),
    ExpectedString(String),
    ExpectedLabel(String),
    InvalidInteger(ParseIntError),
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> ParseError {
        ParseError::InvalidInteger(e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parses(code: &str, expected: Assembly) {
        let parser = Parser::new();
        let assembly = parser.parse(code).unwrap();

        assert_eq!(assembly, expected);
    }

    #[test]
    fn empty_input() {
        assert_parses("", Assembly::new());
    }

    #[test]
    fn single_instruction() {
        assert_parses(
            "Noop",
            Assembly::new()
                .with_section(Section::unnamed().with_instruction(InstructionKind::Noop)),
        );
    }

    #[test]
    fn multiple_instructions() {
        assert_parses(
            r#"
            Noop
            Noop
            Noop
            "#,
            Assembly::new().with_section(
                Section::unnamed()
                    .with_instruction(InstructionKind::Noop)
                    .with_instruction(InstructionKind::Noop)
                    .with_instruction(InstructionKind::Noop),
            ),
        );
    }

    #[test]
    fn labelled_section() {
        assert_parses(
            r#"
            @This_is_a_label_name!
                Noop
                Noop
                Noop
            "#,
            Assembly::new().with_section(
                Section::named("This_is_a_label_name!")
                    .with_instruction(InstructionKind::Noop)
                    .with_instruction(InstructionKind::Noop)
                    .with_instruction(InstructionKind::Noop),
            ),
        );
    }

    #[test]
    fn multiple_sections() {
        assert_parses(
            r#"
            Noop
            Noop

            @This_is_a_label_name!
                Noop
                Noop
                Noop
            "#,
            Assembly::new()
                .with_section(
                    Section::unnamed()
                        .with_instruction(InstructionKind::Noop)
                        .with_instruction(InstructionKind::Noop),
                )
                .with_section(
                    Section::named("This_is_a_label_name!")
                        .with_instruction(InstructionKind::Noop)
                        .with_instruction(InstructionKind::Noop)
                        .with_instruction(InstructionKind::Noop),
                ),
        );
    }

    #[test]
    fn comments() {
        assert_parses(
            r#"
            ; This is a comment on an instruction
            Noop
            Noop

            ; This is a comment on a section
            @This_is_a_label_name!
                Noop
                Noop
                Noop
            "#,
            Assembly::new()
                .with_section(
                    Section::unnamed()
                        .with_commented_instruction(
                            "This is a comment on an instruction",
                            InstructionKind::Noop,
                        )
                        .with_instruction(InstructionKind::Noop),
                )
                .with_section(
                    Section::named("This_is_a_label_name!")
                        .with_comment("This is a comment on a section")
                        .with_instruction(InstructionKind::Noop)
                        .with_instruction(InstructionKind::Noop)
                        .with_instruction(InstructionKind::Noop),
                ),
        );
    }
}
