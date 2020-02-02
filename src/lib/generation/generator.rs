use crate::generation::*;
use crate::semantics::*;
use crate::syntax::*;
// use crate::vm::NativeMethod;
use crate::assembly::*;
use crate::*;
// use num_traits::ToPrimitive;
// use std::collections::hash_map::DefaultHasher;
// use std::hash::{Hash, Hasher};

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
    locals: Vec<Id>,
}

impl<'a> Generator<'a> {
    pub fn new(analysis: &'a mut Analysis) -> Generator<'a> {
        Generator {
            analysis,
            locals: vec![],
        }
    }

    /*
    fn behaviour_id(&self, method: &Node) -> Option<u64> {
        let mut hasher = DefaultHasher::new();
        let selector = self.analysis.navigator.method_selector(&method)?;
        selector.hash(&mut hasher);
        Some(hasher.finish())
    }
    */

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
            if let Class { .. } = declaration.kind {
                self.declare_class(assembly, declaration)?;
            }
        }

        Ok(())
        /*
        let mut instructions = Instructions::new();
        for declaration in declarations.iter() {
            if let Class { .. } = declaration.kind {
                instructions.extend(self.declare_class(declaration)?);
            }
        }
        for declaration in declarations.iter() {
            match declaration.kind {
                Class { .. } => {
                    instructions.extend(self.generate_class(declaration)?);
                }
                LetBinding { .. } => {
                    instructions.extend(self.generate_let_binding(declaration)?);
                    instructions.push(Instruction::StoreGlobal(declaration.id));
                    instructions.push(Instruction::DropLocal(self.index_of_local(&declaration)?));
                    self.local_ids
                        .retain(|existing_local| *existing_local != declaration.id);
                    self.local_count -= 1;
                }
                _ => return Err(invalid_node(declaration, "Expected declaration.")),
            }
        }
        for declaration in declarations.iter() {
            if let Class { .. } = declaration.kind {
                instructions.extend(self.resolve_inherits(declaration)?);
            }
        }
        Ok(instructions)
        */
    }

    fn declare_class(&mut self, assembly: &mut Assembly, class: &Node) -> GenerationResult<()> {
        let (qn, _, _) = self.analysis.navigator.qualified_name_of(class)?;
        let mut section = Section::named(qn.as_str());
        section.add_instruction(InstructionKind::DeclareClass(qn.clone()));

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

        if let Some(body) = self.analysis.navigator.method_body(method) {
            self.generate_expression(assembly, &mut method_section, &body)?;

            method_section.add_instruction(InstructionKind::Return(
                self.analysis.navigator.method_arity(method)? as u16,
            ));
        }

        assembly.add_section(method_section);
        Ok((selector, label))
    }

    fn generate_expression(
        &mut self,
        assembly: &mut Assembly,
        section: &mut Section,
        expression: &Node,
    ) -> GenerationResult<()> {
        match expression.kind {
            SelfExpression(_) => {
                section.add_instruction(InstructionKind::LoadLocal(self.locals.len() as u16));
            }
            StringExpression(_, ref v) => {
                section.add_instruction(InstructionKind::LoadConstString(v.clone()));
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
                    _ => return Err(invalid_node(&declaration, "Expected value declaration.")),
                }
            }
            MessageSendExpression {
                expression: r,
                message: _,
            } => {
                let behaviour = self
                    .analysis
                    .types
                    .get_behaviour_from_message_send(expression)?;

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

    fn generate_number(&mut self, literal: &Node) -> GenerationResult {
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
                "Loa/Number" => {
                    if let IntegerExpression(_, _) = literal.kind {
                        return self.generate_int(literal, BitSize::SizeBig, true);
                    } else {
                        return self.generate_float(literal, BitSize::SizeBig);
                    }
                }
                "Loa/Integer" => return self.generate_int(literal, BitSize::SizeBig, true),
                "Loa/Natural" => return self.generate_int(literal, BitSize::SizeBig, false),
                "Loa/Float" => return self.generate_float(literal, BitSize::SizeBig),

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

    fn generate_int(&self, literal: &Node, size: BitSize, signed: bool) -> GenerationResult {
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

    fn generate_float(&self, literal: &Node, size: BitSize) -> GenerationResult {
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

    fn get_native_method(&self, class: &Node, method: &Node) -> Option<NativeMethod> {
        let (class, _, _) = self.analysis.navigator.qualified_name_of(class)?;
        let selector = self.analysis.navigator.method_selector(method)?;

        Some(format!("{}#{}", class, selector).as_str().into())
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
}
