use crate::assembly::*;
use crate::generation::*;
use crate::semantics::*;
use crate::syntax::*;
use crate::*;
use num_traits::ToPrimitive;

pub type GenerationResult<T> = Result<T, GenerationError>;

pub trait REPLDirectives {
    fn show_type(&self, type_: Type);
    fn show_behaviours(&self, type_: Type, types: &Types);
}

impl REPLDirectives for () {
    fn show_type(&self, _: Type) {}
    fn show_behaviours(&self, _: Type, _: &Types) {}
}

pub struct Generator<'a> {
    analysis: &'a mut Analysis,
    locals: Stack<Id>,
    parameters: Vec<Id>,
}

impl<'a> Generator<'a> {
    pub fn new(analysis: &'a mut Analysis) -> Generator<'a> {
        Generator {
            analysis,
            locals: Stack::new(),
            parameters: vec![],
        }
    }

    pub fn generate_all(&mut self) -> GenerationResult<Assembly> {
        let mut assembly = Assembly::new();
        for source in self.analysis.navigator.sources() {
            if let SourceKind::Module = source.kind {
                self.generate_source((), &mut assembly, &source.uri)?;
            }
        }
        for source in self.analysis.navigator.sources() {
            if let SourceKind::REPLLine = source.kind {
                self.generate_source((), &mut assembly, &source.uri)?;
            }
        }
        assembly
            .last_leading_mut()
            .add_instruction(InstructionKind::Halt);

        Ok(assembly)
    }

    pub fn generate<D: REPLDirectives>(
        &mut self,
        directives: D,
        assembly: &mut Assembly,
        uri: &URI,
    ) -> GenerationResult<()> {
        self.generate_source(directives, assembly, uri)?;

        assembly
            .last_leading_mut()
            .add_instruction(InstructionKind::Halt);

        Ok(())
    }

    fn generate_source<D: REPLDirectives>(
        &mut self,
        directives: D,
        assembly: &mut Assembly,
        uri: &URI,
    ) -> GenerationResult<()> {
        let root = self.analysis.navigator.root_of(uri)?;

        match root.kind {
            Module { .. } => self.generate_module(assembly, &root)?,
            REPLLine { .. } => self.generate_repl_line(directives, assembly, &root)?,
            _ => return Err(invalid_node(&root, "Module or REPLLine expected.")),
        }

        Ok(())
    }

    fn generate_module(&mut self, assembly: &mut Assembly, module: &Node) -> GenerationResult<()> {
        self.generate_declarations(
            assembly,
            &self
                .analysis
                .navigator
                .module_declarations_in(module)
                .into_iter()
                .map(|(_, n)| n)
                .collect(),
        )
    }

    fn generate_declarations(
        &mut self,
        assembly: &mut Assembly,
        declarations: &Vec<Node>,
    ) -> GenerationResult<()> {
        for declaration in declarations.iter() {
            match declaration.kind {
                Class { .. } => self.declare_class(assembly, declaration)?,
                LetBinding { .. } => {
                    self.declare_global_let_binding(assembly, declaration)?;
                }
                _ => return Err(invalid_node(&declaration, "Expected declaration.")),
            }
        }

        Ok(())
    }

    fn declare_global_let_binding(
        &mut self,
        assembly: &mut Assembly,
        binding: &Node,
    ) -> GenerationResult<()> {
        match binding.kind {
            LetBinding { expression, .. } => {
                let label = self.analysis.navigator.symbol_of(binding)?.0;
                let mut section = Section::named(label.clone());
                let expression = self.analysis.navigator.find_child(binding, expression)?;
                self.generate_expression(assembly, &mut section, &expression)?;
                section.add_instruction(InstructionKind::StoreGlobal(label.clone()));
                assembly.add_leading_section(section);
                Ok(())
            }
            _ => Err(invalid_node(binding, "Expected let binding.")),
        }
    }

    fn declare_let_binding(
        &mut self,
        assembly: &mut Assembly,
        section: &mut Section,
        binding: &Node,
    ) -> GenerationResult<()> {
        match binding.kind {
            LetBinding { expression, .. } => {
                let expression = self.analysis.navigator.find_child(binding, expression)?;
                self.generate_expression(assembly, section, &expression)?;
                self.locals.push(binding.id);
                Ok(())
            }
            _ => Err(invalid_node(binding, "Expected let binding.")),
        }
    }

    fn declare_class(&mut self, assembly: &mut Assembly, class: &Node) -> GenerationResult<()> {
        let (qn, _, _) = self.analysis.navigator.qualified_name_of(class)?;
        let mut section = Section::named(qn.as_str());
        section.add_instruction(InstructionKind::DeclareClass(qn.clone()));

        if class.span.start.uri.is_stdlib() {
            let qn = qn.clone();
            match qn.as_str() {
                "Loa/String" => section.add_instruction(InstructionKind::MarkClassString(qn)),
                "Loa/Character" => section.add_instruction(InstructionKind::MarkClassCharacter(qn)),
                "Loa/Symbol" => section.add_instruction(InstructionKind::MarkClassSymbol(qn)),

                "Loa/UInt8" => section.add_instruction(InstructionKind::MarkClassU8(qn)),
                "Loa/UInt16" => section.add_instruction(InstructionKind::MarkClassU16(qn)),
                "Loa/UInt32" => section.add_instruction(InstructionKind::MarkClassU32(qn)),
                "Loa/UInt64" => section.add_instruction(InstructionKind::MarkClassU64(qn)),
                "Loa/UInt128" => section.add_instruction(InstructionKind::MarkClassU128(qn)),
                "Loa/BigNatural" => section.add_instruction(InstructionKind::MarkClassUBig(qn)),
                "Loa/Int8" => section.add_instruction(InstructionKind::MarkClassI8(qn)),
                "Loa/Int16" => section.add_instruction(InstructionKind::MarkClassI16(qn)),
                "Loa/Int32" => section.add_instruction(InstructionKind::MarkClassI32(qn)),
                "Loa/Int64" => section.add_instruction(InstructionKind::MarkClassI64(qn)),
                "Loa/Int128" => section.add_instruction(InstructionKind::MarkClassI128(qn)),
                "Loa/BigInteger" => section.add_instruction(InstructionKind::MarkClassIBig(qn)),
                "Loa/Float32" => section.add_instruction(InstructionKind::MarkClassF32(qn)),
                "Loa/Float64" => section.add_instruction(InstructionKind::MarkClassF64(qn)),
                "Loa/BigFloat" => section.add_instruction(InstructionKind::MarkClassFBig(qn)),
                _ => {}
            }
        }

        for method in self.analysis.navigator.methods_of_class(class) {
            let (method_name, method_label) =
                self.declare_method(assembly, qn.as_ref(), &method)?;

            section.add_instruction(InstructionKind::DeclareMethod(method_name, method_label));
        }

        for behaviour in self
            .analysis
            .types
            .get_behaviours(&self.analysis.types.get_type_of_declaration(class))
        {
            let method = self.analysis.navigator.find_node(behaviour.method_id)?;
            let owning_class = self.analysis.navigator.closest_class_upwards(&method)?;

            if owning_class.id != class.id {
                let (owning_class_qn, _, _) =
                    self.analysis.navigator.qualified_name_of(&owning_class)?;
                let selector = self.analysis.navigator.method_selector(&method)?;
                let label = format!("{}#{}", owning_class_qn, selector);

                section.add_instruction(InstructionKind::DeclareMethod(selector, label));
            }
        }

        assembly.add_leading_section(section);
        Ok(())
    }

    fn declare_method(
        &mut self,
        assembly: &mut Assembly,
        class_name: &str,
        method: &Node,
    ) -> GenerationResult<(String, Label)> {
        let selector = self.analysis.navigator.method_selector(method)?;
        let label = format!("{}#{}", class_name, selector);
        let mut method_section = Section::named(label.as_str());
        if self.analysis.navigator.method_is_native(method) {
            method_section.add_instruction(InstructionKind::CallNative(label.as_str().into()));
            method_section.add_instruction(InstructionKind::Return(0));
        } else if let Some(body) = self.analysis.navigator.method_body(method) {
            for param in self.analysis.navigator.method_parameters(method) {
                self.parameters.push(param.id);
            }

            self.generate_expression(assembly, &mut method_section, &body)?;

            method_section.add_instruction(InstructionKind::Return(
                self.analysis.navigator.method_arity(method)? as u16,
            ));

            self.parameters.clear();
        } else {
            method_section.add_instruction(InstructionKind::LoadConstString(format!("{} is not implemented.", label)));
            method_section.add_instruction(InstructionKind::Panic);
        }

        assembly.add_section(method_section);
        Ok((selector, label))
    }

    fn index_of_parameter(&self, parameter: Id) -> GenerationResult<u16> {
        for (i, p) in self.parameters.iter().enumerate() {
            if *p == parameter {
                return Ok((i + 1 + self.locals.size()) as u16);
            }
        }
        Err(GenerationError::OutOfScope(parameter))
    }

    fn index_of_local(&self, local: Id) -> GenerationResult<u16> {
        for (i, l) in self.locals.iter().enumerate() {
            if *l == local {
                return Ok(i as u16);
            }
        }
        Err(GenerationError::OutOfScope(local))
    }

    fn generate_expression(
        &mut self,
        assembly: &mut Assembly,
        section: &mut Section,
        expression: &Node,
    ) -> GenerationResult<()> {
        match expression.kind {
            SelfExpression(_) => {
                section.add_instruction(InstructionKind::LoadLocal(self.locals.size() as u16));
            }
            PanicExpression{expression: e, ..} => {
                let e = self.analysis.navigator.find_child(expression, e)?;
                self.generate_expression(assembly, section, &e)?;
                section.add_instruction(InstructionKind::Panic);
            }
            StringExpression(_, ref v) => {
                section.add_instruction(InstructionKind::LoadConstString(v.clone()));
            }
            CharacterExpression(_, ref v) => {
                section.add_instruction(InstructionKind::LoadConstCharacter(v.unwrap().clone()));
            }
            SymbolExpression(_, ref v) => {
                section.add_instruction(InstructionKind::LoadConstSymbol(v.clone()));
            }
            IntegerExpression(_, _) | FloatExpression(_, _) => {
                self.generate_number(section, expression)?;
            }
            LetExpression {
                let_binding,
                expression: e,
                ..
            } => {
                let let_binding = self
                    .analysis
                    .navigator
                    .find_child(expression, let_binding)?;
                let expression = self.analysis.navigator.find_child(expression, e)?;
                self.declare_let_binding(assembly, section, &let_binding)?;
                self.generate_expression(assembly, section, &expression)?;
                let index = self.index_of_local(let_binding.id)?;
                section.add_instruction(InstructionKind::DropLocal(index + 1));
                self.locals.drop(index as usize);
            }
            ReferenceExpression { .. } => {
                let declaration = self
                    .analysis
                    .navigator
                    .find_declaration(expression, DeclarationKind::Value)?;
                match declaration.kind {
                    Class { .. } => {
                        let (qn, _, _) = self.analysis.navigator.qualified_name_of(&declaration)?;
                        section.add_instruction(InstructionKind::LoadObject(qn));
                    }
                    ParameterPattern { .. } => section.add_instruction(InstructionKind::LoadLocal(
                        self.index_of_parameter(declaration.id)?,
                    )),
                    LetBinding { .. } => match self.index_of_local(declaration.id) {
                        Ok(index) => section.add_instruction(InstructionKind::LoadLocal(index)),
                        _ => section.add_instruction(InstructionKind::LoadGlobal(
                            self.analysis.navigator.symbol_of(&declaration)?.0,
                        )),
                    },

                    _ => return Err(invalid_node(&declaration, "Expected value declaration.")),
                }
            }
            MessageSendExpression {
                expression: r,
                message,
            } => {
                let behaviour = self
                    .analysis
                    .types
                    .get_behaviour_from_message_send(expression)?;

                // Arguments
                let message = self.analysis.navigator.find_child(expression, message)?;
                let arguments = self.analysis.navigator.message_arguments(&message);
                for argument in arguments.iter() {
                    self.generate_expression(assembly, section, argument)?;
                    self.locals.push(argument.id);
                }

                // Receiver
                let expression = self.analysis.navigator.find_child(expression, r)?;
                self.generate_expression(assembly, section, &expression)?;

                let qualified_name = match behaviour.receiver_type {
                    Type::Class(_, class, _) => {
                        let class = self.analysis.navigator.find_node(class)?;
                        let (qn, _, _) = self.analysis.navigator.qualified_name_of(&class)?;
                        qn
                    }
                    ref t => t.to_string(),
                };

                let label = format!("{}#{}", qualified_name, behaviour.selector());
                let Location {
                    ref uri,
                    line,
                    character,
                    ..
                } = expression.span.start;
                section.add_instruction(InstructionKind::CallMethod(
                    label,
                    uri.to_string(),
                    line as u64,
                    character as u64,
                ));
                for _ in arguments {
                    self.locals.pop();
                }
            }
            _ => return Err(invalid_node(expression, "Expected expression.")),
        }
        Ok(())
    }

    fn generate_repl_line<D: REPLDirectives>(
        &mut self,
        directives: D,
        assembly: &mut Assembly,
        repl_line: &Node,
    ) -> GenerationResult<()> {
        match repl_line.kind {
            REPLLine { ref statements } => {
                for statement in statements.iter() {
                    let statement = self.analysis.navigator.find_child(repl_line, *statement)?;
                    self.generate_repl_statement(&directives, assembly, &statement)?;
                }
                Ok(())
            }
            _ => Err(invalid_node(repl_line, "REPLLine expected.")),
        }
    }

    fn generate_repl_statement<D: REPLDirectives>(
        &mut self,
        directives: &D,
        assembly: &mut Assembly,
        repl_statement: &Node,
    ) -> GenerationResult<()> {
        match repl_statement.kind {
            REPLExpression { expression, .. } => {
                let expression = self
                    .analysis
                    .navigator
                    .find_child(&repl_statement, expression)?;
                let mut section = Section::unnamed();
                self.generate_expression(assembly, &mut section, &expression)?;
                assembly.add_leading_section(section);
                Ok(())
            }
            REPLDirective {
                symbol, expression, ..
            } => {
                let symbol = self.analysis.navigator.find_child(repl_statement, symbol)?;
                if let syntax::Symbol(ref token) = symbol.kind {
                    match token.lexeme().as_ref() {
                        "t" | "type" => {
                            let expression = self
                                .analysis
                                .navigator
                                .find_child(repl_statement, expression)?;
                            let type_ = self.analysis.types.get_type_of_expression(&expression);
                            directives.show_type(type_);
                        }
                        "b" | "behaviours" => {
                            let expression = self
                                .analysis
                                .navigator
                                .find_child(repl_statement, expression)?;
                            let type_ = self.analysis.types.get_type_of_expression(&expression);
                            directives.show_behaviours(type_, &self.analysis.types);
                        }
                        _ => return Err(invalid_node(&symbol, "Invalid REPL directive.")),
                    }
                }
                Ok(())
            }
            ImportDirective { .. } => Ok(()),
            _ => self.generate_declarations(assembly, &vec![repl_statement.clone()]),
        }
    }
    /*

    fn index_of_local(&self, declaration: &Node) -> Option<u16> {
        self.local_ids
            .iter()
            .position(|id| *id == declaration.id)
            .map(|i| i as u16)
    }

    fn generate_expression(&mut self, expression: &Node) -> GenerationResult {
        let result = match expression.kind {
            ReferenceExpression { .. } => {
                let declaration = self
                    .analysis
                    .navigator
                    .find_declaration(expression, DeclarationKind::Value)?;

                match declaration.kind {
                    Class { .. } => Ok(Instruction::ReferenceToClass(declaration.id).into()),
                    ParameterPattern { .. } => Ok(Instruction::LoadLocal(
                        (self.analysis.navigator.index_of_parameter(&declaration)?) as u16
                            + 1 // self
                            + self.local_count,
                    )
                    .into()),
                    LetBinding { .. } => Ok(match self.index_of_local(&declaration) {
                        Some(idx) => Instruction::LoadLocal(idx),
                        None => Instruction::LoadGlobal(declaration.id),
                    }
                    .into()),
                    _ => Err(invalid_node(&declaration, "Expected declaration.")),
                }
            }

            StringExpression(_, _) => self.generate_string(expression),

            CharacterExpression(_, _) => self.generate_character(expression),

            IntegerExpression(_, _) | FloatExpression(_, _) => self.generate_number(expression),

            SymbolExpression(_, ref s) => Ok(Instruction::LoadConstSymbol(s.clone()).into()),

            SelfExpression(_) => Ok(Instruction::LoadLocal(self.local_count).into()),

            LetExpression {
                let_binding,
                expression: eid,
                ..
            } => {
                let let_binding = self
                    .analysis
                    .navigator
                    .find_child(expression, let_binding)?;
                let expression = self.analysis.navigator.find_child(expression, eid)?;

                let mut instructions = Instructions::new();
                instructions.extend(self.generate_let_binding(&let_binding)?);
                instructions.extend(self.generate_expression(&expression)?);
                Ok(instructions)
            }

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
                instructions.push(Instruction::SendMessage(
                    format!("{}", expression.span.start),
                    self.behaviour_id(&method)?,
                ));

                self.local_count -= self.analysis.navigator.message_arity(&message)? as u16;

                Ok(instructions)
            }

            _ => Err(invalid_node(expression, "Expected expression.")),
        };

        // This expression will be pushed to the stack,
        // which increases the number of locals.
        self.local_count += 1;

        result
    }

    fn generate_string(&mut self, string: &Node) -> GenerationResult {
        match string.kind {
            StringExpression(_, ref s) => Ok(Instruction::LoadConstString(s.clone()).into()),
            _ => Err(invalid_node(string, "Expected string.")),
        }
    }

    fn generate_character(&mut self, character: &Node) -> GenerationResult {
        match character.kind {
            CharacterExpression(_, Some(ref s)) => {
                Ok(Instruction::LoadConstCharacter(s.clone()).into())
            }
            _ => Err(invalid_node(character, "Expected string.")),
        }
    }

    */
    fn generate_number(&mut self, section: &mut Section, literal: &Node) -> GenerationResult<()> {
        let type_ = self.analysis.types.get_type_of_expression(&literal);

        if let Type::UnresolvedInteger(_, _) = type_ {
            return self.generate_int(section, literal, BitSize::SizeBig, true);
        }

        if let Type::UnresolvedFloat(_, _) = type_ {
            return self.generate_float(section, literal, BitSize::SizeBig);
        }

        if let Type::Class(_, class, _) = type_ {
            let class = self.analysis.navigator.find_node(class)?;
            let (qn, _, _) = self.analysis.navigator.qualified_name_of(&class)?;

            match qn.as_str() {
                "Loa/Number" => {
                    if let IntegerExpression(_, _) = literal.kind {
                        return self.generate_int(section, literal, BitSize::SizeBig, true);
                    } else {
                        return self.generate_float(section, literal, BitSize::SizeBig);
                    }
                }
                "Loa/Integer" => {
                    return self.generate_int(section, literal, BitSize::SizeBig, true)
                }
                "Loa/Natural" => {
                    return self.generate_int(section, literal, BitSize::SizeBig, false)
                }
                "Loa/Float" => return self.generate_float(section, literal, BitSize::SizeBig),

                "Loa/Int8" => return self.generate_int(section, literal, BitSize::Size8, true),
                "Loa/Int16" => return self.generate_int(section, literal, BitSize::Size16, true),
                "Loa/Int32" => return self.generate_int(section, literal, BitSize::Size32, true),
                "Loa/Int64" => return self.generate_int(section, literal, BitSize::Size64, true),
                "Loa/Int128" => return self.generate_int(section, literal, BitSize::Size128, true),
                "Loa/BigInteger" => {
                    return self.generate_int(section, literal, BitSize::SizeBig, true)
                }
                "Loa/UInt8" => return self.generate_int(section, literal, BitSize::Size8, false),
                "Loa/UInt16" => return self.generate_int(section, literal, BitSize::Size16, false),
                "Loa/UInt32" => return self.generate_int(section, literal, BitSize::Size32, false),
                "Loa/UInt64" => return self.generate_int(section, literal, BitSize::Size64, false),
                "Loa/UInt128" => {
                    return self.generate_int(section, literal, BitSize::Size128, false)
                }
                "Loa/BigNatural" => {
                    return self.generate_int(section, literal, BitSize::SizeBig, false)
                }
                "Loa/Float32" => return self.generate_float(section, literal, BitSize::Size32),
                "Loa/Float64" => return self.generate_float(section, literal, BitSize::Size64),
                "Loa/BigFloat" => return self.generate_float(section, literal, BitSize::SizeBig),
                _ => (),
            }
        }
        Err(invalid_node(
            literal,
            format!("Invalid type for a literal: {}", type_).as_ref(),
        ))
    }

    fn generate_int(
        &self,
        section: &mut Section,
        literal: &Node,
        size: BitSize,
        signed: bool,
    ) -> GenerationResult<()> {
        section.add_instruction(match (&literal.kind, signed) {
            (IntegerExpression(_, ref int), true) => match size {
                BitSize::Size8 => InstructionKind::LoadConstI8(int.to_i8().unwrap()).into(),
                BitSize::Size16 => InstructionKind::LoadConstI16(int.to_i16().unwrap()).into(),
                BitSize::Size32 => InstructionKind::LoadConstI32(int.to_i32().unwrap()).into(),
                BitSize::Size64 => InstructionKind::LoadConstI64(int.to_i64().unwrap()).into(),
                BitSize::Size128 => InstructionKind::LoadConstI128(int.to_i128().unwrap()).into(),
                BitSize::SizeBig => InstructionKind::LoadConstIBig(int.clone()).into(),
            },
            (IntegerExpression(_, ref int), false) => match size {
                BitSize::Size8 => InstructionKind::LoadConstU8(int.to_u8().unwrap()).into(),
                BitSize::Size16 => InstructionKind::LoadConstU16(int.to_u16().unwrap()).into(),
                BitSize::Size32 => InstructionKind::LoadConstU32(int.to_u32().unwrap()).into(),
                BitSize::Size64 => InstructionKind::LoadConstU64(int.to_u64().unwrap()).into(),
                BitSize::Size128 => InstructionKind::LoadConstU128(int.to_u128().unwrap()).into(),
                BitSize::SizeBig => {
                    InstructionKind::LoadConstUBig(int.to_biguint().unwrap()).into()
                }
            },
            _ => return Err(invalid_node(literal, "Expected integer expression.")),
        });
        Ok(())
    }

    fn generate_float(
        &self,
        section: &mut Section,
        literal: &Node,
        size: BitSize,
    ) -> GenerationResult<()> {
        section.add_instruction(match literal.kind {
            FloatExpression(_, ref fraction) => match size {
                BitSize::Size32 => InstructionKind::LoadConstF32(fraction.to_f32().unwrap()).into(),
                BitSize::Size64 => InstructionKind::LoadConstF64(fraction.to_f64().unwrap()).into(),
                BitSize::SizeBig => InstructionKind::LoadConstFBig(fraction.clone()).into(),
                _ => return Err(invalid_node(literal, "Invalid bit size of float.")),
            },
            _ => return Err(invalid_node(literal, "Expected float expression.")),
        });
        Ok(())
    }

    /*
    fn generate_message(&mut self, message: &Node) -> GenerationResult {
        match message.kind {
            UnaryMessage { .. } => Ok(Instructions::new()),
            BinaryMessage { expression, .. } => {
                let expression = self.analysis.navigator.find_child(message, expression)?;

                self.generate_expression(&expression)
            }
            KeywordMessage { ref keyword_pairs } => {
                let mut instructions = Instructions::new();

                for pair in keyword_pairs.iter().rev() {
                    let pair = self.analysis.navigator.find_child(message, *pair)?;

                    match pair.kind {
                        KeywordPair { value, .. } => {
                            let expression = self.analysis.navigator.find_child(&pair, value)?;

                            instructions.extend(self.generate_expression(&expression)?);
                        }
                        _ => return Err(invalid_node(&pair, "Expected keyword pair.")),
                    }
                }

                Ok(instructions)
            }
            _ => Err(invalid_node(message, "Expected expression.")),
        }
    }

    fn resolve_inherits(&mut self, class: &Node) -> GenerationResult {
        let mut instructions = Instructions::new();
        let class_type = self.analysis.types.get_type_of_declaration(&class);
        let behaviours = self.analysis.types.get_behaviours(&class_type);

        for behaviour in behaviours {
            let method = self.analysis.navigator.find_node(behaviour.method_id)?;

            if behaviour.receiver_type == class_type {
                // Noop
            } else if let Type::Class(_, class_id, _) = behaviour.receiver_type {
                instructions.push(Instruction::InheritMethod(
                    class_id,
                    class.id,
                    self.behaviour_id(&method)?,
                ));
            }
        }

        Ok(instructions)
    }

    fn declare_class(&mut self, class: &Node) -> GenerationResult {
        let (name, _, _) = self.analysis.navigator.qualified_name_of(class)?;
        let mut instructions = Instructions::new();

        instructions.push(Instruction::DeclareClass(class.id, name.clone()));

        if class.span.start.uri.is_stdlib() {
            match name.as_str() {
                "Loa/String" => instructions.push(Instruction::MarkClassString(class.id)),
                "Loa/Character" => instructions.push(Instruction::MarkClassCharacter(class.id)),
                "Loa/Symbol" => instructions.push(Instruction::MarkClassSymbol(class.id)),

                "Loa/UInt8" => instructions.push(Instruction::MarkClassU8(class.id)),
                "Loa/UInt16" => instructions.push(Instruction::MarkClassU16(class.id)),
                "Loa/UInt32" => instructions.push(Instruction::MarkClassU32(class.id)),
                "Loa/UInt64" => instructions.push(Instruction::MarkClassU64(class.id)),
                "Loa/UInt128" => instructions.push(Instruction::MarkClassU128(class.id)),
                "Loa/BigNatural" => instructions.push(Instruction::MarkClassUBig(class.id)),
                "Loa/Int8" => instructions.push(Instruction::MarkClassI8(class.id)),
                "Loa/Int16" => instructions.push(Instruction::MarkClassI16(class.id)),
                "Loa/Int32" => instructions.push(Instruction::MarkClassI32(class.id)),
                "Loa/Int64" => instructions.push(Instruction::MarkClassI64(class.id)),
                "Loa/Int128" => instructions.push(Instruction::MarkClassI128(class.id)),
                "Loa/BigInteger" => instructions.push(Instruction::MarkClassIBig(class.id)),
                "Loa/Float32" => instructions.push(Instruction::MarkClassF32(class.id)),
                "Loa/Float64" => instructions.push(Instruction::MarkClassF64(class.id)),
                "Loa/BigFloat" => instructions.push(Instruction::MarkClassFBig(class.id)),
                _ => {}
            }
        }

        Ok(instructions)
    }

    fn generate_class(&mut self, class: &Node) -> GenerationResult {
        let mut instructions = Instructions::new();
        let class_type = self.analysis.types.get_type_of_declaration(&class);
        let behaviours = self.analysis.types.get_behaviours(&class_type);

        for behaviour in behaviours {
            let method = self.analysis.navigator.find_node(behaviour.method_id)?;

            if behaviour.receiver_type == class_type {
                instructions.extend(self.generate_method(class, &method)?);
            }
        }

        Ok(instructions)
    }

    fn generate_method(&mut self, class: &Node, method: &Node) -> GenerationResult {
        self.local_count = 0;
        self.local_ids.clear();

        let mut instructions = Instructions::new();
        match method.kind {
            Method {
                ref native_keyword,
                signature,
                method_body,
                ..
            } => {
                if native_keyword.is_none() && method_body == Id::NULL {
                    return Ok(vec![].into());
                }

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
                        instructions.push(Instruction::BeginMethod(
                            self.behaviour_id(method)?,
                            selector,
                        ));
                        instructions.extend(self.generate_message_pattern(&message_pattern)?);
                    }
                    _ => return Err(invalid_node(&signature, "Expected signature.")),
                }

                if native_keyword.is_some() {
                    instructions.push(Instruction::CallNative(
                        self.get_native_method(class, method)?,
                    ));
                } else if let Some(method_body) =
                    self.analysis.navigator.find_child(method, method_body)
                {
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
                        self.analysis.navigator.method_arity(method)? as u16 + self.local_count,
                    ));
                }
                instructions.push(Instruction::EndMethod(class.id));
            }
            _ => return Err(invalid_node(method, "Expected method.")),
        }

        Ok(instructions)
    }

    fn generate_message_pattern(&mut self, message_pattern: &Node) -> GenerationResult {
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

    fn generate_parameter_pattern(
        &mut self,
        _parameter_pattern: &Node,
        _arity: usize,
    ) -> GenerationResult {
        Ok(Instructions::new())
        // Ok(Instruction::LoadArgument(arity).into())
    }

    fn generate_let_binding(&mut self, binding: &Node) -> GenerationResult {
        match binding.kind {
            LetBinding { expression, .. } => {
                let expression = self.analysis.navigator.find_child(binding, expression)?;
                self.local_ids.insert(0, binding.id);
                self.generate_expression(&expression)
            }
            _ => Err(invalid_node(binding, "Expected let binding.")),
        }
    }
    */
}

fn invalid_node(node: &Node, message: &str) -> GenerationError {
    GenerationError::InvalidNode(node.clone(), message.into())
}

#[cfg(test)]
mod tests {
    use crate::assembly::Parser as AssemblyParser;
    use crate::generation::*;
    use crate::semantics::Analysis;
    use crate::syntax::Parser;
    use crate::*;

    fn assert_generates(source: Arc<Source>, expected: &str) {
        let mut analysis = Analysis::new(Arc::new(
            vec![(source.uri.clone(), Parser::new(source).parse().0)]
                .into_iter()
                .collect(),
        ));
        let mut generator = Generator::new(&mut analysis);
        let assembly = generator.generate_all().unwrap();

        let expected = AssemblyParser::new().parse(expected).unwrap();

        assert_eq!(assembly, expected);
    }

    #[test]
    fn empty_class() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class C.
                "#,
            ),
            r#"
                @N/C
                    DeclareClass "N/C"
                    Halt
            "#,
        );
    }

    #[test]
    fn simple_method() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class C {
                        public x => self.
                    }
                "#,
            ),
            r#"
                @N/C
                    DeclareClass "N/C"
                    DeclareMethod "x" @N/C#x
                    Halt

                @N/C#x
                    LoadLocal 0
                    Return 1
            "#,
        );
    }

    #[test]
    fn inherited_method() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class A {
                        public x => self.
                    }

                    class B {
                        is A.
                    }
                "#,
            ),
            r#"
                @N/A
                    DeclareClass "N/A"
                    DeclareMethod "x" @N/A#x

                @N/B
                    DeclareClass "N/B"
                    DeclareMethod "x" @N/A#x
                    Halt

                @N/A#x
                    LoadLocal 0
                    Return 1
            "#,
        );
    }

    #[test]
    fn constant_string() {
        assert_generates(
            Source::test_repl(
                r#"
                    "hello"
                "#,
            ),
            r#"
                LoadConstString "hello"
                Halt
            "#,
        );
    }

    #[test]
    fn unary_message() {
        assert_generates(
            Source::test_repl(
                r#"
                    class A {
                        public x => self.
                    }

                    A x.
                "#,
            ),
            r#"
                @A
                    DeclareClass "A"
                    DeclareMethod "x" @A#x

                LoadObject @A
                CallMethod @A#x "test:" 6 21
                Halt

                @A#x
                    LoadLocal 0
                    Return 1
            "#,
        );
    }

    #[test]
    fn unary_message_send_to_self() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class A {
                        public x => self y.
                        public y => self.
                    }
                "#,
            ),
            r#"
                @N/A
                    DeclareClass "N/A"
                    DeclareMethod "x" @N/A#x
                    DeclareMethod "y" @N/A#y
                    Halt

                @N/A#x
                    LoadLocal 0
                    CallMethod @N/A#y "test:" 5 37
                    Return 1

                @N/A#y
                    LoadLocal 0
                    Return 1
            "#,
        );
    }

    #[test]
    fn binary_method() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class A {
                        public + A other => other.
                    }
                "#,
            ),
            r#"
                @N/A
                    DeclareClass "N/A"
                    DeclareMethod "+" @N/A#+
                    Halt

                @N/A#+
                    LoadLocal 1
                    Return 2
            "#,
        );
    }

    #[test]
    fn local_binding() {
        assert_generates(
            Source::test_repl(
                r#"
                    let x = "string".
                    x
                "#,
            ),
            r#"
                @x
                    LoadConstString "string"
                    StoreGlobal @x

                LoadGlobal @x
                Halt
            "#,
        );
    }

    #[test]
    fn local_binding_in_method() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class X {
                        public y =>
                            let la = "one".
                            let lb = "two".
                            la.
                    }
                "#,
            ),
            r#"
                @N/X
                    DeclareClass "N/X"
                    DeclareMethod "y" @N/X#y
                    Halt

                @N/X#y
                    LoadConstString "one"
                    LoadConstString "two"
                    LoadLocal 1
                    DropLocal 1
                    DropLocal 1
                    Return 1
            "#,
        );
    }

    #[test]
    fn native_method() {
        assert_generates(
            Source::test(
                r#"
                    namespace Loa.

                    class Number {
                        public native + Number -> Number.
                    }
                "#,
            ),
            r#"
                @Loa/Number
                    DeclareClass "Loa/Number"
                    DeclareMethod "+" @Loa/Number#+
                    Halt

                @Loa/Number#+
                    CallNative Number_plus
                    Return 0
            "#,
        );
    }

    #[test]
    fn binary_call() {
        assert_generates(
            Source::test_repl(
                r#"
                    class A {
                        public + A other => other.
                    }

                    A + A.
                "#,
            ),
            r#"
                @A
                    DeclareClass "A"
                    DeclareMethod "+" @A#+

                ; Right-hand operand
                LoadObject @A

                ; Left-hand operand (receiver)
                LoadObject @A

                CallMethod @A#+ "test:" 6 21
                Halt

                @A#+
                    LoadLocal 1
                    Return 2
            "#,
        );
    }

    #[test]
    fn local_as_argument() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class A {
                        public go =>
                          let a = A.
                          a + a.

                        public + A other => other.
                    }
                "#,
            ),
            r#"
                @N/A
                    DeclareClass "N/A"
                    DeclareMethod "go" @N/A#go
                    DeclareMethod "+" @N/A#+
                    Halt

                @N/A#go
                    ; let a = A.
                    LoadObject @N/A

                    ; RHS
                    LoadLocal 0

                    ; LHS – because the stack grew with last instruction
                    LoadLocal 1

                    CallMethod @N/A#+ "test:" 7 27
                    DropLocal 1
                    Return 1

                @N/A#+
                    LoadLocal 1
                    Return 2
            "#,
        );
    }
}
