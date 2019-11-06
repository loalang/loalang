use crate::generation::*;
use crate::semantics::*;
use crate::syntax::*;
use crate::*;
use num_traits::ToPrimitive;

pub type GenerationResult = Result<Instructions, GenerationError>;

pub trait REPLDirectives {
    fn show_type(type_: Type);
    fn show_behaviours(type_: Type, types: &Types);
}

impl REPLDirectives for () {
    fn show_type(_: Type) {}
    fn show_behaviours(_: Type, _: &Types) {}
}

pub struct Generator<'a> {
    analysis: &'a mut Analysis,
    local_count: u16,
}

impl<'a> Generator<'a> {
    pub fn new(analysis: &'a mut Analysis) -> Generator<'a> {
        Generator {
            analysis,
            local_count: 0,
        }
    }

    pub fn generate<D: REPLDirectives>(&mut self, uri: &URI) -> GenerationResult {
        let root = self.analysis.navigator.root_of(uri)?;

        match root.kind {
            Module { .. } => self.generate_module(&root),
            REPLLine { .. } => self.generate_repl_line::<D>(&root),
            _ => Err(invalid_node(&root, "Module or REPLLine expected.")),
        }
    }

    pub fn generate_module(&mut self, module: &Node) -> GenerationResult {
        match module.kind {
            Module {
                ref module_declarations,
                ..
            } => {
                let mut instructions = Instructions::new();
                for dec in module_declarations {
                    let dec = self.analysis.navigator.find_child(module, *dec)?;
                    instructions.extend(self.generate_module_declaration(&dec)?);
                }
                Ok(instructions)
            }
            _ => Err(invalid_node(module, "Expected module.")),
        }
    }

    pub fn generate_module_declaration(&mut self, module_declaration: &Node) -> GenerationResult {
        match module_declaration.kind {
            Exported(_, declaration) => {
                let declaration = self
                    .analysis
                    .navigator
                    .find_child(&module_declaration, declaration)?;
                self.generate_declaration(&declaration)
            }
            _ => self.generate_declaration(&module_declaration),
        }
    }

    pub fn generate_repl_line<D: REPLDirectives>(&mut self, repl_line: &Node) -> GenerationResult {
        match repl_line.kind {
            REPLLine { ref statements } => {
                let mut instructions = Instructions::new();
                for statement in statements.iter() {
                    let statement = self.analysis.navigator.find_child(repl_line, *statement)?;
                    instructions.extend(self.generate_repl_statement::<D>(&statement)?);
                }
                Ok(instructions)
            }
            _ => Err(invalid_node(repl_line, "REPLLine expected.")),
        }
    }

    pub fn generate_repl_statement<D: REPLDirectives>(
        &mut self,
        repl_statement: &Node,
    ) -> GenerationResult {
        match repl_statement.kind {
            REPLExpression { expression, .. } => {
                let expression = self
                    .analysis
                    .navigator
                    .find_child(&repl_statement, expression)?;
                self.generate_expression(&expression)
            }
            REPLDirective {
                symbol, expression, ..
            } => {
                let symbol = self.analysis.navigator.find_child(repl_statement, symbol)?;
                let expression = self
                    .analysis
                    .navigator
                    .find_child(repl_statement, expression)?;
                if let syntax::Symbol(ref token) = symbol.kind {
                    match token.lexeme().as_ref() {
                        "t" | "type" => {
                            let type_ = self.analysis.types.get_type_of_expression(&expression);
                            D::show_type(type_);
                        }
                        "b" | "behaviours" => {
                            let type_ = self.analysis.types.get_type_of_expression(&expression);
                            D::show_behaviours(type_, &self.analysis.types);
                        }
                        _ => return Err(invalid_node(&symbol, "Invalid REPL directive.")),
                    }
                }
                Ok(vec![].into())
            }
            ImportDirective { .. } => Ok(vec![].into()),
            _ => self.generate_declaration(&repl_statement),
        }
    }

    pub fn generate_expression(&mut self, expression: &Node) -> GenerationResult {
        match expression.kind {
            ReferenceExpression { .. } => {
                let declaration = self
                    .analysis
                    .navigator
                    .find_declaration(expression, DeclarationKind::Value)?;

                let result = match declaration.kind {
                    Class { .. } => Ok(Instruction::ReferenceToClass(declaration.id).into()),
                    ParameterPattern { .. } => Ok(Instruction::LoadLocal(
                        (self.analysis.navigator.index_of_parameter(&declaration)?) as u16
                            + self.local_count,
                    )
                    .into()),
                    _ => Err(invalid_node(&declaration, "Expected declaration.")),
                };

                // This expression will be pushed to the stack,
                // which increases the number of locals.
                self.local_count += 1;

                result
            }

            IntegerExpression(_, _) | FloatExpression(_, _) => self.generate_literal(expression),

            SelfExpression(_) => Ok(Instruction::LoadLocal(self.local_count - 1).into()),

            MessageSendExpression {
                expression: receiver,
                message,
                ..
            } => {
                let receiver = self.analysis.navigator.find_child(expression, receiver)?;
                let message = self.analysis.navigator.find_child(expression, message)?;
                let method = self
                    .analysis
                    .navigator
                    .method_from_message(&message, &self.analysis.types)?;

                let mut instructions = Instructions::new();

                instructions.extend(self.generate_message(&message)?);
                instructions.extend(self.generate_expression(&receiver)?);
                instructions.push(Instruction::SendMessage(method.id));

                Ok(instructions)
            }

            _ => Err(invalid_node(expression, "Expected expression.")),
        }
    }

    pub fn generate_literal(&mut self, literal: &Node) -> GenerationResult {
        let type_ = self.analysis.types.get_type_of_expression(&literal);

        if let Type::UnresolvedInteger(_, _) = type_ {
            return self.generate_int(literal, BitSize::SizeBig, true);
        }

        if let Type::UnresolvedFloat(_, _) = type_ {
            return self.generate_float(literal, BitSize::SizeBig);
        }

        if let Type::Class(_, class, _) = type_ {
            let class = self.analysis.navigator.find_node(class)?;
            let (qn, _, _) = self.analysis.navigator.qualified_name_of(&class)?;

            match qn.as_str() {
                "Loa/Int8" => return self.generate_int(literal, BitSize::Size8, true),
                "Loa/Int16" => return self.generate_int(literal, BitSize::Size16, true),
                "Loa/Int32" => return self.generate_int(literal, BitSize::Size32, true),
                "Loa/Int64" => return self.generate_int(literal, BitSize::Size64, true),
                "Loa/Int128" => return self.generate_int(literal, BitSize::Size128, true),
                "Loa/BigInteger" => return self.generate_int(literal, BitSize::SizeBig, true),
                "Loa/UInt8" => return self.generate_int(literal, BitSize::Size8, false),
                "Loa/UInt16" => return self.generate_int(literal, BitSize::Size16, false),
                "Loa/UInt32" => return self.generate_int(literal, BitSize::Size32, false),
                "Loa/UInt64" => return self.generate_int(literal, BitSize::Size64, false),
                "Loa/UInt128" => return self.generate_int(literal, BitSize::Size128, false),
                "Loa/BigNatural" => return self.generate_int(literal, BitSize::SizeBig, false),
                "Loa/Float32" => return self.generate_float(literal, BitSize::Size32),
                "Loa/Float64" => return self.generate_float(literal, BitSize::Size64),
                "Loa/BigFloat" => return self.generate_float(literal, BitSize::SizeBig),
                _ => (),
            }
        }
        Err(invalid_node(
            literal,
            format!("Invalid type for a literal: {}", type_).as_ref(),
        ))
    }

    pub fn generate_int(&self, literal: &Node, size: BitSize, signed: bool) -> GenerationResult {
        match (&literal.kind, signed) {
            (IntegerExpression(_, ref int), true) => match size {
                BitSize::Size8 => Ok(Instruction::LoadConstI8(int.to_i8().unwrap()).into()),
                BitSize::Size16 => Ok(Instruction::LoadConstI16(int.to_i16().unwrap()).into()),
                BitSize::Size32 => Ok(Instruction::LoadConstI32(int.to_i32().unwrap()).into()),
                BitSize::Size64 => Ok(Instruction::LoadConstI64(int.to_i64().unwrap()).into()),
                BitSize::Size128 => Ok(Instruction::LoadConstI128(int.to_i128().unwrap()).into()),
                BitSize::SizeBig => Ok(Instruction::LoadConstIBig(int.clone()).into()),
            },
            (IntegerExpression(_, ref int), false) => match size {
                BitSize::Size8 => Ok(Instruction::LoadConstU8(int.to_u8().unwrap()).into()),
                BitSize::Size16 => Ok(Instruction::LoadConstU16(int.to_u16().unwrap()).into()),
                BitSize::Size32 => Ok(Instruction::LoadConstU32(int.to_u32().unwrap()).into()),
                BitSize::Size64 => Ok(Instruction::LoadConstU64(int.to_u64().unwrap()).into()),
                BitSize::Size128 => Ok(Instruction::LoadConstU128(int.to_u128().unwrap()).into()),
                BitSize::SizeBig => {
                    Ok(Instruction::LoadConstUBig(int.to_biguint().unwrap()).into())
                }
            },
            _ => Err(invalid_node(literal, "Expected integer expression.")),
        }
    }

    pub fn generate_float(&self, literal: &Node, size: BitSize) -> GenerationResult {
        match literal.kind {
            FloatExpression(_, ref fraction) => match size {
                BitSize::Size32 => Ok(Instruction::LoadConstF32(fraction.to_f32().unwrap()).into()),
                BitSize::Size64 => Ok(Instruction::LoadConstF64(fraction.to_f64().unwrap()).into()),
                BitSize::SizeBig => Ok(Instruction::LoadConstFBig(fraction.clone()).into()),
                _ => Err(invalid_node(literal, "Invalid bit size of float.")),
            },
            _ => Err(invalid_node(literal, "Expected float expression.")),
        }
    }

    pub fn generate_message(&mut self, message: &Node) -> GenerationResult {
        match message.kind {
            UnaryMessage { .. } => Ok(Instructions::new()),
            BinaryMessage { expression, .. } => {
                let expression = self.analysis.navigator.find_child(message, expression)?;

                self.generate_expression(&expression)
            }
            KeywordMessage { ref keyword_pairs } => {
                let mut instructions = Instructions::new();

                for pair in keyword_pairs.iter() {
                    let pair = self.analysis.navigator.find_child(message, *pair)?;

                    match pair.kind {
                        KeywordPair { value, .. } => {
                            let expression = self.analysis.navigator.find_child(&pair, value)?;

                            instructions.extend(self.generate_expression(&expression)?);
                        }
                        _ => return Err(invalid_node(&pair, "Expected keyword pair.")),
                    }
                }

                instructions.reverse();

                Ok(instructions)
            }
            _ => Err(invalid_node(message, "Expected expression.")),
        }
    }

    pub fn generate_declaration(&mut self, declaration: &Node) -> GenerationResult {
        match declaration.kind {
            Class { .. } => {
                let (name, _) = self.analysis.navigator.symbol_of(declaration)?;
                let methods = self.analysis.navigator.methods_of_class(declaration);
                let mut instructions = Instructions::new();

                instructions.push(Instruction::DeclareClass(declaration.id, name));

                for method in methods {
                    instructions.extend(self.generate_method(&method)?);
                }

                Ok(instructions)
            }
            _ => Err(invalid_node(declaration, "Expected declaration.")),
        }
    }

    pub fn generate_method(&mut self, method: &Node) -> GenerationResult {
        // The first local is `self`
        self.local_count = 1;

        let mut instructions = Instructions::new();
        match method.kind {
            Method {
                signature,
                method_body,
                ..
            } => {
                let signature = self.analysis.navigator.find_child(method, signature)?;
                match signature.kind {
                    Signature {
                        message_pattern, ..
                    } => {
                        let message_pattern = self
                            .analysis
                            .navigator
                            .find_child(&signature, message_pattern)?;
                        let selector = self
                            .analysis
                            .navigator
                            .message_pattern_selector(&message_pattern)?;
                        instructions.push(Instruction::BeginMethod(selector));
                        instructions.extend(self.generate_message_pattern(&message_pattern)?);
                    }
                    _ => return Err(invalid_node(&signature, "Expected signature.")),
                }

                if let Some(method_body) = self.analysis.navigator.find_child(method, method_body) {
                    match method_body.kind {
                        MethodBody { expression, .. } => {
                            let expression = self
                                .analysis
                                .navigator
                                .find_child(&method_body, expression)?;
                            instructions.extend(self.generate_expression(&expression)?);
                        }
                        _ => return Err(invalid_node(&method_body, "Expected method body.")),
                    }

                    instructions.push(Instruction::Return(
                        self.analysis.navigator.method_arity(method)? as u8,
                    ));
                }
                instructions.push(Instruction::EndMethod(method.id));
            }
            _ => return Err(invalid_node(method, "Expected method.")),
        }

        Ok(instructions)
    }

    pub fn generate_message_pattern(&mut self, message_pattern: &Node) -> GenerationResult {
        let mut instructions = Instructions::new();
        match message_pattern.kind {
            UnaryMessagePattern { .. } => {}
            BinaryMessagePattern {
                parameter_pattern, ..
            } => {
                let parameter_pattern = self
                    .analysis
                    .navigator
                    .find_child(message_pattern, parameter_pattern)?;

                instructions.extend(self.generate_parameter_pattern(&parameter_pattern, 2)?);
            }
            KeywordMessagePattern {
                ref keyword_pairs, ..
            } => {
                let mut pairs = keyword_pairs.clone();
                pairs.reverse();
                let arity = pairs.len() + 1;
                for pair in pairs {
                    let pair = self.analysis.navigator.find_child(message_pattern, pair)?;
                    match pair.kind {
                        KeywordPair { value, .. } => {
                            let parameter_pattern =
                                self.analysis.navigator.find_child(&pair, value)?;

                            instructions.extend(
                                self.generate_parameter_pattern(&parameter_pattern, arity)?,
                            );
                        }
                        _ => return Err(invalid_node(&pair, "Expected keyword pair.")),
                    }
                }
            }
            _ => return Err(invalid_node(message_pattern, "Expected message pattern.")),
        }

        Ok(instructions)
    }

    pub fn generate_parameter_pattern(
        &mut self,
        _parameter_pattern: &Node,
        _arity: usize,
    ) -> GenerationResult {
        Ok(Instructions::new())
        // Ok(Instruction::LoadArgument(arity).into())
    }
}

fn invalid_node(node: &Node, message: &str) -> GenerationError {
    GenerationError::InvalidNode(node.clone(), message.into())
}

pub enum BitSize {
    Size8,
    Size16,
    Size32,
    Size64,
    Size128,
    SizeBig,
}
