use crate::assembly::*;
use crate::vm::NativeMethod;
use std::num::{ParseFloatError, ParseIntError};

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
            // DumpStack
            else if code.starts_with("DumpStack") {
                code.drain(.."DumpStack".len());
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::DumpStack,
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
                let line = self.parse_from_str(code)?;
                let character = self.parse_from_str(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::CallMethod(label, uri, line, character),
                });
            }
            // CallNative <native method>
            else if code.starts_with("CallNative") {
                code.drain(.."CallNative".len());
                let method = self.parse_native_method(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::CallNative(method),
                });
            }
            // LoadLocal <u16>
            else if code.starts_with("LoadLocal") {
                code.drain(.."LoadLocal".len());
                let index = self.parse_from_str(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::LoadLocal(index),
                });
            }
            // DropLocal <u16>
            else if code.starts_with("DropLocal") {
                code.drain(.."DropLocal".len());
                let index = self.parse_from_str(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::DropLocal(index),
                });
            }
            // StoreGlobal <label>
            else if code.starts_with("StoreGlobal") {
                code.drain(.."StoreGlobal".len());
                let label = self.parse_label(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::StoreGlobal(label),
                });
            }
            // LoadGlobal <label>
            else if code.starts_with("LoadGlobal") {
                code.drain(.."LoadGlobal".len());
                let label = self.parse_label(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::LoadGlobal(label),
                });
            }
            // Return <u16>
            else if code.starts_with("Return") {
                code.drain(.."Return".len());
                let arity = self.parse_from_str(code)?;
                section.instructions.push(Instruction {
                    leading_comment,
                    kind: InstructionKind::Return(arity),
                });
            }
            // MarkClass..
            else if code.starts_with("MarkClass") {
                code.drain(.."MarkClass".len());
                // ..String <label>
                if code.starts_with("String") {
                    code.drain(.."String".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassString(label),
                    });
                }
                // ..Character <label>
                else if code.starts_with("Character") {
                    code.drain(.."Character".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassCharacter(label),
                    });
                }
                // ..Symbol <label>
                else if code.starts_with("Symbol") {
                    code.drain(.."Symbol".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassSymbol(label),
                    });
                }
                // ..U8 <label>
                else if code.starts_with("U8") {
                    code.drain(.."U8".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassU8(label),
                    });
                }
                // ..U16 <label>
                else if code.starts_with("U16") {
                    code.drain(.."U16".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassU16(label),
                    });
                }
                // ..U32 <label>
                else if code.starts_with("U32") {
                    code.drain(.."U32".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassU32(label),
                    });
                }
                // ..u64 <label>
                else if code.starts_with("U64") {
                    code.drain(.."U64".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassU64(label),
                    });
                }
                // ..U128 <label>
                else if code.starts_with("U128") {
                    code.drain(.."U128".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassU128(label),
                    });
                }
                // ..UBig <label>
                else if code.starts_with("UBig") {
                    code.drain(.."UBig".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassUBig(label),
                    });
                }
                // ..i8 <label>
                else if code.starts_with("I8") {
                    code.drain(.."I8".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassI8(label),
                    });
                }
                // ..I16 <label>
                else if code.starts_with("I16") {
                    code.drain(.."I16".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassI16(label),
                    });
                }
                // ..I32 <label>
                else if code.starts_with("I32") {
                    code.drain(.."I32".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassI32(label),
                    });
                }
                // ..I64 <label>
                else if code.starts_with("I64") {
                    code.drain(.."I64".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassI64(label),
                    });
                }
                // ..I128 <label>
                else if code.starts_with("I128") {
                    code.drain(.."I128".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassI128(label),
                    });
                }
                // ..IBig <label>
                else if code.starts_with("IBig") {
                    code.drain(.."IBig".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassIBig(label),
                    });
                }
                // ..F32 <label>
                else if code.starts_with("F32") {
                    code.drain(.."F32".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassF32(label),
                    });
                }
                // ..F64 <label>
                else if code.starts_with("F64") {
                    code.drain(.."F64".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassF64(label),
                    });
                }
                // ..FBig <label>
                else if code.starts_with("FBig") {
                    code.drain(.."FBig".len());
                    let label = self.parse_label(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::MarkClassFBig(label),
                    });
                }
                // ..
                else {
                    return Err(ParseError::ExpectedConstTag(code.clone()));
                }
            }
            // LoadConst..
            else if code.starts_with("LoadConst") {
                code.drain(.."LoadConst".len());
                // ..String <string>
                if code.starts_with("String") {
                    code.drain(.."String".len());
                    let value = self.parse_string(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstString(value),
                    });
                }
                // ..Character <character>
                else if code.starts_with("Character") {
                    code.drain(.."Character".len());
                    let value = self.parse_character(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstCharacter(value),
                    });
                }
                // ..Symbol <symbol>
                else if code.starts_with("Symbol") {
                    code.drain(.."Symbol".len());
                    let value = self.parse_symbol(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstSymbol(value),
                    });
                }
                // ..U8 <integer>
                else if code.starts_with("U8") {
                    code.drain(.."U8".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstU8(value),
                    });
                }
                // ..U16 <integer>
                else if code.starts_with("U16") {
                    code.drain(.."U16".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstU16(value),
                    });
                }
                // ..U32 <integer>
                else if code.starts_with("U32") {
                    code.drain(.."U32".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstU32(value),
                    });
                }
                // ..u64 <integer>
                else if code.starts_with("U64") {
                    code.drain(.."U64".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstU64(value),
                    });
                }
                // ..U128 <integer>
                else if code.starts_with("U128") {
                    code.drain(.."U128".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstU128(value),
                    });
                }
                // ..UBig <integer>
                else if code.starts_with("UBig") {
                    code.drain(.."UBig".len());
                    let value: u128 = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstUBig(value.into()),
                    });
                }
                // ..i8 <integer>
                else if code.starts_with("I8") {
                    code.drain(.."I8".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstI8(value),
                    });
                }
                // ..I16 <integer>
                else if code.starts_with("I16") {
                    code.drain(.."I16".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstI16(value),
                    });
                }
                // ..I32 <integer>
                else if code.starts_with("I32") {
                    code.drain(.."I32".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstI32(value),
                    });
                }
                // ..I64 <integer>
                else if code.starts_with("I64") {
                    code.drain(.."I64".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstI64(value),
                    });
                }
                // ..I128 <integer>
                else if code.starts_with("I128") {
                    code.drain(.."I128".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstI128(value),
                    });
                }
                // ..IBig <integer>
                else if code.starts_with("IBig") {
                    code.drain(.."IBig".len());
                    let value: i128 = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstIBig(value.into()),
                    });
                }
                // ..F32 <float>
                else if code.starts_with("F32") {
                    code.drain(.."F32".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstF32(value),
                    });
                }
                // ..F64 <float>
                else if code.starts_with("F64") {
                    code.drain(.."F64".len());
                    let value = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstF64(value),
                    });
                }
                // ..FBig <float>
                else if code.starts_with("FBig") {
                    code.drain(.."FBig".len());
                    let value: f64 = self.parse_from_str(code)?;
                    section.instructions.push(Instruction {
                        leading_comment,
                        kind: InstructionKind::LoadConstFBig(value.into()),
                    });
                }
                // ..
                else {
                    return Err(ParseError::ExpectedConstTag(code.clone()));
                }
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

    fn parse_native_method(&mut self, code: &mut String) -> ParseResult<NativeMethod> {
        self.skip_leading_whitespace(code);
        if code.starts_with("Number_plus") {
            code.drain(.."Number_plus".len());
            Ok(NativeMethod::Number_plus)
        } else {
            Err(ParseError::ExpectedNativeMethod(code.clone()))
        }
    }

    fn parse_character(&mut self, code: &mut String) -> ParseResult<u16> {
        self.skip_leading_whitespace(code);
        if !code.starts_with("'") {
            return Err(ParseError::ExpectedString(code.clone()));
        }
        code.drain(.."'".len());
        let mut result = String::new();
        while code.len() > 0 && !code.starts_with("'") {
            result.push(code.remove(0));
        }
        if code.starts_with("'") {
            code.drain(.."'".len());
        }
        Ok(crate::syntax::string_to_characters(result)[0])
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

    fn parse_symbol(&mut self, code: &mut String) -> ParseResult<String> {
        self.skip_leading_whitespace(code);
        if !code.starts_with("#") {
            return Err(ParseError::ExpectedLabel(code.clone()));
        }
        code.drain(.."#".len());
        let mut symbol = String::new();
        while code.len() > 0 && !(code.as_bytes()[0] as char).is_whitespace() {
            symbol.push(code.remove(0));
        }
        Ok(symbol)
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

    fn parse_from_str<P: std::str::FromStr>(&mut self, code: &mut String) -> ParseResult<P>
    where
        P::Err: Into<ParseError>,
    {
        self.skip_leading_whitespace(code);
        let mut number = String::new();
        while code.len() > 0 && !(code.as_bytes()[0] as char).is_whitespace() {
            number.push(code.remove(0));
        }
        Ok(number.parse().map_err(Into::into)?)
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    ExpectedInstruction(String),
    ExpectedConstTag(String),
    ExpectedString(String),
    ExpectedLabel(String),
    ExpectedNativeMethod(String),
    InvalidInteger(ParseIntError),
    InvalidFloat(ParseFloatError),
}

impl From<ParseIntError> for ParseError {
    fn from(e: ParseIntError) -> ParseError {
        ParseError::InvalidInteger(e)
    }
}

impl From<ParseFloatError> for ParseError {
    fn from(e: ParseFloatError) -> ParseError {
        ParseError::InvalidFloat(e)
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
