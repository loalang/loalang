use crate::assembly::*;
use std::num::ParseIntError;

pub struct Parser {
    indent_size: usize,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { indent_size: 0 }
    }

    pub fn parse(&mut self, code: &str) -> ParseResult<Assembly> {
        let mut assembly = Assembly::new();
        let mut code = String::from(code.trim_start());
        self.skip_leading_whitespace(&mut code);
        while !code.is_empty() {
            assembly.add_section(self.parse_section(&mut code)?);
            self.skip_leading_whitespace(&mut code);
        }
        Ok(assembly)
    }

    fn skip_leading_whitespace(&mut self, s: &mut String) {
        let mut is_in_indent = self.indent_size > 0;
        loop {
            if s.len() == 0 {
                break;
            }

            let next = &(s.as_bytes()[0] as char);

            match next {
                ' ' | '\t' => {
                    s.drain(..1);
                    if is_in_indent {
                        self.indent_size += 1;
                    }
                }
                '\n' => {
                    s.drain(..1);
                    self.indent_size = 0;
                    is_in_indent = true;
                }
                _ => {
                    break;
                }
            }
        }
    }

    fn parse_section(&mut self, code: &mut String) -> ParseResult<Section> {
        let mut section = Section::unnamed();

        if code.starts_with(";") {
            section.leading_comment = Some(self.parse_comment(code)?);
        }

        self.skip_leading_whitespace(code);
        if code.starts_with("@") {
            section.label = Some(self.parse_label(code)?);
        }

        self.skip_leading_whitespace(code);
        let indent_before = self.indent_size;
        while !code.is_empty() {
            let mut leading_comment = None;

            if code.starts_with(";") {
                leading_comment = Some(self.parse_comment(code)?);
            }

            if self.indent_size < indent_before || code.starts_with("@") {
                break;
            }
            // Noop
            else if code.starts_with("Noop") {
                code.drain(.."Noop".len());
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Noop,
                });
            }
            // Halt
            else if code.starts_with("Halt") {
                code.drain(.."Halt".len());
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Halt,
                });
            }
            // Panic
            else if code.starts_with("Panic") {
                code.drain(.."Panic".len());
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Panic,
                });
            }
            // DeclareClass <string>
            else if code.starts_with("DeclareClass") {
                code.drain(.."DeclareClass".len());
                let name = self.parse_string(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::DeclareClass(name),
                });
            }
            // DeclareMethod <string> <label>
            else if code.starts_with("DeclareMethod") {
                code.drain(.."DeclareMethod".len());
                let name = self.parse_string(code)?;
                let label = self.parse_label(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::DeclareMethod(name, label),
                });
            }
            // LoadObject <label>
            else if code.starts_with("LoadObject") {
                code.drain(.."LoadObject".len());
                let label = self.parse_label(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::LoadObject(label),
                });
            }
            // CallMethod <label> <string> <u46> <u64>
            else if code.starts_with("CallMethod") {
                code.drain(.."CallMethod".len());
                let label = self.parse_label(code)?;
                let uri = self.parse_string(code)?;
                let line = self.parse_u64(code)?;
                let character = self.parse_u64(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::CallMethod(label, uri, line, character),
                });
            }
            // LoadLocal <u16>
            else if code.starts_with("LoadLocal") {
                code.drain(.."LoadLocal".len());
                let index = self.parse_u16(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::LoadLocal(index),
                });
            }
            // Return <u16>
            else if code.starts_with("Return") {
                code.drain(.."Return".len());
                let arity = self.parse_u16(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Return(arity),
                });
            }
            // LoadConstString <string>
            else if code.starts_with("LoadConstString") {
                code.drain(.."LoadConstString".len());
                let value = self.parse_string(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::LoadConstString(value),
                });
            }
            // ...
            else {
                return Err(ParseError::ExpectedInstruction(code.clone()));
            }
            self.skip_leading_whitespace(code);
        }

        Ok(section)
    }

    fn parse_string(&mut self, code: &mut String) -> ParseResult<String> {
        self.skip_leading_whitespace(code);
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

    fn parse_comment(&mut self, code: &mut String) -> ParseResult<String> {
        let mut result = String::new();
        while code.starts_with(";") {
            code.drain(..";".len());
            while code.len() > 0 && !code.starts_with("\n") {
                result.push(code.remove(0));
            }
            self.skip_leading_whitespace(code);
        }
        Ok(result)
    }

    fn parse_label(&mut self, code: &mut String) -> ParseResult<Label> {
        self.skip_leading_whitespace(code);
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

    fn parse_u16(&mut self, code: &mut String) -> ParseResult<u16> {
        self.skip_leading_whitespace(code);
        let mut number = String::new();
        while code.len() > 0 && (code.as_bytes()[0] as char).is_numeric() {
            number.push(code.remove(0));
        }
        Ok(number.parse()?)
    }

    fn parse_u64(&mut self, code: &mut String) -> ParseResult<u64> {
        self.skip_leading_whitespace(code);
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
        let mut parser = Parser::new();
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

    #[test]
    fn unnamed_section_after_named() {
        assert_parses(
            r#"
            ; This is a comment on a section
            @This_is_a_label_name!
                Noop
                Noop
                Noop

            ; This is a comment on an instruction
            Noop
            Noop
            "#,
            Assembly::new()
                .with_section(
                    Section::named("This_is_a_label_name!")
                        .with_comment("This is a comment on a section")
                        .with_instruction(InstructionKind::Noop)
                        .with_instruction(InstructionKind::Noop)
                        .with_instruction(InstructionKind::Noop),
                )
                .with_section(
                    Section::unnamed()
                        .with_commented_instruction(
                            "This is a comment on an instruction",
                            InstructionKind::Noop,
                        )
                        .with_instruction(InstructionKind::Noop),
                ),
        );
    }
}
