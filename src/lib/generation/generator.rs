use crate::assembly::*;
use crate::generation::*;
use crate::semantics::*;
use crate::syntax::*;
use crate::*;
use num_traits::ToPrimitive;

pub type GenerationResult<T> = Result<T, GenerationError>;

enum SimulatedStackElement {
    Self_,
    Declaration(Id),
    Expression(Id),
}

impl SimulatedStackElement {
    pub fn is(&self, id: Id) -> bool {
        match self {
            SimulatedStackElement::Self_ => false,
            SimulatedStackElement::Expression(i) => *i == id,
            SimulatedStackElement::Declaration(i) => *i == id,
        }
    }
}

struct SimulatedStack(Vec<SimulatedStackElement>);

impl SimulatedStack {
    pub fn new() -> SimulatedStack {
        SimulatedStack(vec![])
    }

    pub fn pop(&mut self) {
        #[allow(unused)]
        let result = self.0.pop();
        #[cfg(debug_assertions)]
        assert!(result.is_some());
    }

    pub fn push_self(&mut self) {
        self.0.push(SimulatedStackElement::Self_);
    }

    pub fn push_declaration(&mut self, id: Id) {
        self.0.push(SimulatedStackElement::Declaration(id));
    }

    pub fn push_expression(&mut self, id: Id) {
        self.0.push(SimulatedStackElement::Expression(id));
    }

    pub fn turn_top_into_declaration(&mut self, id: Id) {
        self.pop();
        self.push_declaration(id);
    }

    pub fn assert_empty(&self) {
        #[cfg(debug_assertions)]
        assert_eq!(self.0.len(), 0);
    }

    pub fn drop_index(&mut self, index: usize) {
        self.0.remove(self.0.len() - 1 - index);
    }

    pub fn drop(&mut self, id: Id) -> GenerationResult<()> {
        let index = self.index_of(id)?;
        self.drop_index(index as usize);
        Ok(())
    }

    pub fn index_of_self(&mut self) -> GenerationResult<u16> {
        let mut index = 0;
        for element in self.0.iter().rev() {
            if matches!(element, SimulatedStackElement::Self_) {
                return Ok(index as u16);
            }
            index += 1;
        }
        Err(GenerationError::OutOfScope(Id::NULL))
    }

    pub fn index_of(&mut self, id: Id) -> GenerationResult<u16> {
        let mut index = 0;
        for element in self.0.iter().rev() {
            if element.is(id) {
                return Ok(index as u16);
            }
            index += 1;
        }
        Err(GenerationError::OutOfScope(id))
    }
}

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
    simulated_stack: SimulatedStack,
    lazies: usize,
}

impl<'a> Generator<'a> {
    pub fn new(analysis: &'a mut Analysis) -> Generator<'a> {
        Generator {
            analysis,
            simulated_stack: SimulatedStack::new(),
            lazies: 0,
        }
    }

    fn sub(&mut self) -> Generator {
        let mut sub = Generator::new(self.analysis);
        sub.lazies = self.lazies;
        sub
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
            .last_main_section_mut()
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
            .last_main_section_mut()
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
                assembly.add_main_section(section);
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
                self.simulated_stack.turn_top_into_declaration(binding.id);
                Ok(())
            }
            _ => Err(invalid_node(binding, "Expected let binding.")),
        }
    }

    fn declare_class_object(
        &mut self,
        name: &str,
        assembly: &mut Assembly,
        class: &Node,
    ) -> GenerationResult<()> {
        let mut section = Section::named(format!("{}$class", name));
        section.add_instruction(InstructionKind::DeclareClass(format!("{} class", name)));

        let mut methods_section = Section::named(format!("{}$class$methods", name));

        for initializer in self.analysis.navigator.initializers_of(class) {
            if let Some(selector) = self.analysis.navigator.initializer_selector(&initializer) {
                self.simulated_stack.assert_empty();

                let label = format!("{}$class#{}", name, selector);
                methods_section
                    .add_instruction(InstructionKind::DeclareMethod(selector, label.clone()));
                section.add_instruction(InstructionKind::UseMethod(label.clone()));
                let mut init_section = Section::named(label);

                for param in self
                    .analysis
                    .navigator
                    .initializer_parameters(&initializer)
                    .iter()
                    .rev()
                {
                    self.simulated_stack.push_declaration(param.id);
                }

                self.simulated_stack.push_self();

                let assignments = self
                    .analysis
                    .navigator
                    .initializer_assignments(&initializer);

                for (_, argument) in assignments.iter() {
                    self.generate_expression(assembly, &mut init_section, &argument)?;
                }

                init_section.add_instruction(InstructionKind::LoadObject(name.into()));
                self.simulated_stack.push_expression(Id::NULL);

                for (v, a) in assignments.iter() {
                    let label = format!("{}#{}:", name, v);
                    let Location {
                        ref uri,
                        line,
                        character,
                        ..
                    } = a.span.start;
                    init_section.add_instruction(InstructionKind::CallMethod(
                        label,
                        uri.to_string(),
                        line as u64,
                        character as u64,
                    ));
                    self.simulated_stack.drop_index(1);
                }

                // Class object
                self.simulated_stack.drop_index(1);

                let arity = self.analysis.navigator.initializer_arity(&initializer)?;
                init_section.add_instruction(InstructionKind::Return(arity as u16));

                for _ in 0..arity {
                    self.simulated_stack.pop();
                }

                self.simulated_stack.assert_empty();

                assembly.add_section(init_section);
            }
        }

        if !methods_section.is_empty() {
            assembly.add_method_declaration_section(methods_section);
        }

        assembly.add_class_declaration_section(section);
        Ok(())
    }

    fn declare_class(&mut self, assembly: &mut Assembly, class: &Node) -> GenerationResult<()> {
        let (qn, _, _) = self.analysis.navigator.qualified_name_of(class)?;

        if self.analysis.navigator.has_class_object(class) {
            self.declare_class_object(qn.as_ref(), assembly, class)?;
        }

        let mut section = Section::named(qn.as_str());
        section.add_instruction(InstructionKind::DeclareClass(qn.clone()));

        let mut methods_section = Section::named(format!("{}$methods", qn));

        for variable in self.analysis.navigator.variables_of_class(class) {
            let (variable_name, _) = self.analysis.navigator.symbol_of(&variable)?;
            let variable_label = format!("{}({})", qn, variable_name);
            let getter_label = format!("{}#{}", qn, variable_name);
            let setter_label = format!("{}#{}:", qn, variable_name);

            assembly.add_section(
                Section::named(variable_label.clone()).with_instruction(InstructionKind::Noop),
            );
            assembly.add_section(
                Section::named(getter_label.clone()).with_instruction(InstructionKind::Noop),
            );
            assembly.add_section(
                Section::named(setter_label.clone()).with_instruction(InstructionKind::Noop),
            );

            methods_section.add_instruction(InstructionKind::DeclareVariable(
                variable_name,
                variable_label.clone(),
                getter_label,
                setter_label,
            ));

            section.add_instruction(InstructionKind::UseVariable(variable_label));
        }

        for method in self.analysis.navigator.methods_of_class(class) {
            let (method_name, method_label) =
                self.declare_method(assembly, qn.as_ref(), &method)?;

            methods_section.add_instruction(InstructionKind::DeclareMethod(
                method_name,
                method_label.clone(),
            ));

            for overridden_method in self.analysis.navigator.methods_overridden_by(&method) {
                let label = self
                    .analysis
                    .navigator
                    .qualified_name_of_method(&overridden_method)?;
                section
                    .add_instruction(InstructionKind::OverrideMethod(label, method_label.clone()));
            }

            section.add_instruction(InstructionKind::UseMethod(method_label));
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

                section.add_instruction(InstructionKind::UseMethod(label));
            }
        }

        if !methods_section.is_empty() {
            assembly.add_method_declaration_section(methods_section);
        }

        if class.span.start.uri.is_stdlib() {
            let qn = qn.clone();
            match qn.as_str() {
                "Loa/True" => section.add_instruction(InstructionKind::MarkClassTrue(qn)),
                "Loa/False" => section.add_instruction(InstructionKind::MarkClassFalse(qn)),

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

        assembly.add_class_declaration_section(section);
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
            self.simulated_stack.assert_empty();

            let parameters = self.analysis.navigator.method_parameters(method);
            for param in parameters.iter().rev() {
                self.simulated_stack.push_declaration(param.id);
            }
            self.simulated_stack.push_self();

            self.generate_expression(assembly, &mut method_section, &body)?;

            method_section.add_instruction(InstructionKind::Return(
                self.analysis.navigator.method_arity(method)? as u16,
            ));
            for _ in parameters {
                self.simulated_stack.pop();
            }
            // Receiver
            self.simulated_stack.pop();

            // Result
            self.simulated_stack.pop();

            self.simulated_stack.assert_empty();
        } else {
            method_section.add_instruction(InstructionKind::LoadConstString(format!(
                "{} is not implemented.",
                label
            )));
            method_section.add_instruction(InstructionKind::Panic);
        }

        assembly.add_section(method_section);
        Ok((selector, label))
    }

    fn generate_reference_to(
        &mut self,
        section: &mut Section,
        declaration: &Node,
    ) -> GenerationResult<()> {
        match declaration.kind {
            Class { .. } => {
                let (qn, _, _) = self.analysis.navigator.qualified_name_of(declaration)?;
                if self.analysis.navigator.has_class_object(declaration) {
                    let qn = format!("{}$class", qn);
                    section.add_instruction(InstructionKind::LoadObject(qn));
                } else {
                    section.add_instruction(InstructionKind::LoadObject(qn));
                }
            }
            ParameterPattern { .. } => section.add_instruction(InstructionKind::LoadLocal(
                self.simulated_stack.index_of(declaration.id)?,
            )),
            LetBinding { .. } => match self.simulated_stack.index_of(declaration.id) {
                Ok(index) => section.add_instruction(InstructionKind::LoadLocal(index)),
                _ => section.add_instruction(InstructionKind::LoadGlobal(
                    self.analysis.navigator.symbol_of(declaration)?.0,
                )),
            },

            _ => return Err(invalid_node(declaration, "Expected value declaration.")),
        }
        Ok(())
    }

    fn qualified_type_name(&self, type_: &Type) -> Option<String> {
        match type_ {
            Type::Class(_, class, _) => {
                let class = self.analysis.navigator.find_node(*class)?;
                let (qn, _, _) = self.analysis.navigator.qualified_name_of(&class)?;
                Some(qn)
            }

            Type::ClassObject(ref t) => self
                .qualified_type_name(t)
                .map(|qn| format!("{}$class", qn)),

            ref t => Some(t.to_string()),
        }
    }

    fn qualified_behaviour_name(&self, behaviour: &Behaviour) -> Option<String> {
        self.qualified_type_name(&behaviour.receiver_type)
    }

    fn generate_expression(
        &mut self,
        assembly: &mut Assembly,
        section: &mut Section,
        expression: &Node,
    ) -> GenerationResult<()> {
        match expression.kind {
            SelfExpression(_) => {
                section.add_instruction(InstructionKind::LoadLocal(
                    self.simulated_stack.index_of_self()?,
                ));
                self.simulated_stack.push_expression(expression.id);
            }
            PanicExpression { expression: e, .. } => {
                let e = self.analysis.navigator.find_child(expression, e)?;
                self.generate_expression(assembly, section, &e)?;
                section.add_instruction(InstructionKind::Panic);
                self.simulated_stack.pop();
            }
            CascadeExpression { expression: e, .. } => {
                let e = self.analysis.navigator.find_child(expression, e)?;
                self.generate_expression(assembly, section, &e)?;
            }
            TupleExpression { expression: e, .. } => {
                let e = self.analysis.navigator.find_child(expression, e)?;
                self.generate_expression(assembly, section, &e)?;
            }
            StringExpression(_, ref v) => {
                section.add_instruction(InstructionKind::LoadConstString(v.clone()));
                self.simulated_stack.push_expression(expression.id);
            }
            CharacterExpression(_, ref v) => {
                section.add_instruction(InstructionKind::LoadConstCharacter(v.unwrap().clone()));
                self.simulated_stack.push_expression(expression.id);
            }
            SymbolExpression(_, ref v) => {
                section.add_instruction(InstructionKind::LoadConstSymbol(v.clone()));
                self.simulated_stack.push_expression(expression.id);
            }
            IntegerExpression(_, _) | FloatExpression(_, _) => {
                self.generate_number(section, expression)?;
                self.simulated_stack.push_expression(expression.id);
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

                let index = self.simulated_stack.index_of(let_binding.id)?;
                section.add_instruction(InstructionKind::DropLocal(index));
                self.simulated_stack.drop(let_binding.id)?;
            }
            ReferenceExpression { .. } => {
                let declaration = self
                    .analysis
                    .navigator
                    .find_declaration(expression, DeclarationKind::Value)?;
                self.generate_reference_to(section, &declaration)?;
                self.simulated_stack.push_expression(expression.id);
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
                for argument in arguments.iter().rev() {
                    match argument.kind {
                        TupleExpression { .. }
                        | MessageSendExpression { .. }
                        | PanicExpression { .. } => {
                            self.generate_lazy(assembly, section, argument)?;
                        }
                        _ => {
                            self.generate_expression(assembly, section, argument)?;
                        }
                    }
                }

                // Receiver
                let expression = self.analysis.navigator.find_child(expression, r)?;
                self.generate_expression(assembly, section, &expression)?;

                let qualified_name = self.qualified_behaviour_name(&behaviour)?;

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
                    self.simulated_stack.pop();
                }
            }
            _ => return Err(invalid_node(expression, "Expected expression.")),
        }
        Ok(())
    }

    fn generate_lazy(
        &mut self,
        assembly: &mut Assembly,
        section: &mut Section,
        expression: &Node,
    ) -> GenerationResult<()> {
        self.lazies += 1;
        let label = format!(
            "{}$lazy{}",
            section.label.as_ref().map(AsRef::as_ref).unwrap_or(""),
            self.lazies
        );

        let arguments = self.analysis.navigator.locals_crossing_into(expression);
        let mut arity = arguments.len() as u16;
        let mut lazy_stack = SimulatedStack::new();
        for argument in arguments.iter() {
            self.generate_reference_to(section, &argument)?;
            lazy_stack.push_declaration(argument.id);
            self.simulated_stack.push_declaration(argument.id);
        }
        if self.analysis.navigator.self_crosses_into(expression) {
            arity += 1;
            section.add_instruction(InstructionKind::LoadLocal(
                self.simulated_stack.index_of_self()?,
            ));
            lazy_stack.push_self();
        }
        section.add_instruction(InstructionKind::LoadLazy(arity, label.clone()));
        for argument in arguments {
            self.simulated_stack.drop(argument.id)?;
        }
        self.simulated_stack.push_expression(expression.id);

        let mut sub_generator = self.sub();
        sub_generator.simulated_stack = lazy_stack;
        let mut lazy_section = Section::named(label);
        sub_generator.generate_expression(assembly, &mut lazy_section, expression)?;
        lazy_section.add_instruction(InstructionKind::ReturnLazy(arity));

        assembly.add_section(lazy_section);
        self.lazies = sub_generator.lazies;

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
                assembly.add_main_section(section);
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
                "Loa/Number" | "Loa/Object" => {
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
                @N/C$methods
                    DeclareMethod "x" @N/C#x

                @N/C
                    DeclareClass "N/C"
                    UseMethod @N/C#x
                
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
                @N/A$methods
                    DeclareMethod "x" @N/A#x

                @N/A
                    DeclareClass "N/A"
                    UseMethod @N/A#x

                @N/B
                    DeclareClass "N/B"
                    UseMethod @N/A#x
                
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
                @A$methods
                    DeclareMethod "x" @A#x

                @A
                    DeclareClass "A"
                    UseMethod @A#x

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
                @N/A$methods
                    DeclareMethod "x" @N/A#x
                    DeclareMethod "y" @N/A#y

                @N/A
                    DeclareClass "N/A"
                    UseMethod @N/A#x
                    UseMethod @N/A#y

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
                @N/A$methods
                    DeclareMethod "+" @N/A#+

                @N/A
                    DeclareClass "N/A"
                    UseMethod @N/A#+

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
                @N/X$methods
                    DeclareMethod "y" @N/X#y

                @N/X
                    DeclareClass "N/X"
                    UseMethod @N/X#y

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
                @Loa/Number$methods
                    DeclareMethod "+" @Loa/Number#+

                @Loa/Number
                    DeclareClass "Loa/Number"
                    UseMethod @Loa/Number#+

                Halt

                @Loa/Number#+
                    CallNative Loa/Number#+
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
                @A$methods
                    DeclareMethod "+" @A#+

                @A
                    DeclareClass "A"
                    UseMethod @A#+

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
                @N/A$methods
                    DeclareMethod "go" @N/A#go
                    DeclareMethod "+" @N/A#+

                @N/A
                    DeclareClass "N/A"
                    UseMethod @N/A#go
                    UseMethod @N/A#+

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

    #[test]
    fn keyword_arguments() {
        assert_generates(
            Source::test_repl(
                r#"
                    class A {
                        public one: A one two: A two => two.
                    }
                    class B {
                        is A.
                    }

                    A one: A two: B.
                "#,
            ),
            r#"
                @A$methods
                    DeclareMethod "one:two:" @A#one:two:

                @A
                    DeclareClass "A"
                    UseMethod @A#one:two:

                @B
                    DeclareClass "B"
                    UseMethod @A#one:two:

                ; two:
                LoadObject @B
                ; one:
                LoadObject @A
                ; self
                LoadObject @A
                CallMethod @A#one:two: "test:" 9 21
                Halt

                @A#one:two:
                    LoadLocal 2
                    Return 3
            "#,
        );
    }

    #[test]
    fn overridden_method() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class A {
                      public x => A.
                    }

                    class B {
                      is A.
                      public x => B.
                    }

                    class C {
                      is A.
                    }
                "#,
            ),
            r#"
                @N/A$methods
                    DeclareMethod "x" @N/A#x

                @N/B$methods
                    DeclareMethod "x" @N/B#x

                @N/A
                    DeclareClass "N/A"
                    UseMethod @N/A#x

                @N/B
                    DeclareClass "N/B"
                    OverrideMethod @N/A#x @N/B#x
                    UseMethod @N/B#x

                @N/C
                    DeclareClass "N/C"
                    UseMethod @N/A#x

                Halt

                @N/A#x
                    LoadObject @N/A
                    Return 1

                @N/B#x
                    LoadObject @N/B
                    Return 1
            "#,
        );
    }

    #[test]
    fn lazy_message_argument() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class A {
                        public yourself => self.

                        public return: A o => o.

                        public main =>
                            self return: A yourself.
                    }
                "#,
            ),
            r#"
                @N/A$methods
                    DeclareMethod "yourself" @N/A#yourself
                    DeclareMethod "return:" @N/A#return:
                    DeclareMethod "main" @N/A#main

                @N/A
                    DeclareClass "N/A"
                    UseMethod @N/A#yourself
                    UseMethod @N/A#return:
                    UseMethod @N/A#main

                Halt

                @N/A#yourself
                    LoadLocal 0
                    Return 1

                @N/A#return:
                    LoadLocal 1
                    Return 2

                @N/A#main$lazy1
                    LoadObject @N/A
                    CallMethod @N/A#yourself "test:" 10 42
                    ReturnLazy 0

                @N/A#main
                    LoadLazy 0 @N/A#main$lazy1
                    LoadLocal 1
                    CallMethod @N/A#return: "test:" 10 29
                    Return 1
            "#,
        );
    }

    #[test]
    fn lazy_with_arguments() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.

                    class A {
                        public yourself => self.

                        public return: A o => o.

                        public main =>
                          let a = A.
                          self return: a yourself.
                    }
                "#,
            ),
            r#"
                @N/A$methods
                    DeclareMethod "yourself" @N/A#yourself
                    DeclareMethod "return:" @N/A#return:
                    DeclareMethod "main" @N/A#main

                @N/A
                    DeclareClass "N/A"
                    UseMethod @N/A#yourself
                    UseMethod @N/A#return:
                    UseMethod @N/A#main

                Halt

                @N/A#yourself
                    LoadLocal 0
                    Return 1

                @N/A#return:
                    LoadLocal 1
                    Return 2

                @N/A#main$lazy1
                    LoadLocal 0
                    CallMethod @N/A#yourself "test:" 11 40
                    ReturnLazy 1

                @N/A#main
                    LoadObject @N/A
                    LoadLocal 0
                    LoadLazy 1 @N/A#main$lazy1
                    LoadLocal 2
                    CallMethod @N/A#return: "test:" 11 27
                    DropLocal 1
                    Return 1
            "#,
        );
    }

    #[test]
    fn unary_initializer() {
        assert_generates(
            Source::test_repl(
                r#"
                    class A {
                        public init new.
                    }

                    A new.
                "#,
            ),
            r#"
                @A$class$methods
                    DeclareMethod "new" @A$class#new

                @A$class
                    DeclareClass "A class"
                    UseMethod @A$class#new

                @A
                    DeclareClass "A"

                LoadObject @A$class
                CallMethod @A$class#new "test:" 6 21
                Halt

                @A$class#new
                    LoadObject @A
                    Return 1
            "#,
        );
    }

    #[test]
    fn single_variable() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.
                    class A {
                        public var B b.
                        public init new: B b =>
                          b: b.
                    }
                    class B.
                "#,
            ),
            r#"
                @N/A$class$methods
                  DeclareMethod "new:" @N/A$class#new:

                @N/A$methods
                  DeclareVariable "b" @N/A(b) @N/A#b @N/A#b:

                @N/A$class
                  DeclareClass "N/A class"
                  UseMethod @N/A$class#new:

                @N/A
                  DeclareClass "N/A"
                  UseVariable @N/A(b)

                @N/B
                  DeclareClass "N/B"

                Halt

                @N/A$class#new:
                  LoadLocal 1
                  LoadObject @N/A
                  CallMethod @N/A#b: "test:" 6 30
                  Return 2

                @N/A(b)
                  Noop

                @N/A#b
                  Noop

                @N/A#b:
                  Noop
            "#,
        );
    }

    #[test]
    fn self_in_lazy() {
        assert_generates(
            Source::test(
                r#"
                    namespace N.
                    class P {
                        public + P p => p.
                    }
                    class A {
                        public a => self p + self p.
                        public p => P.
                    }
                "#,
            ),
            r#"
				@N/P$methods
				  DeclareMethod "+" @N/P#+

				@N/A$methods
				  DeclareMethod "a" @N/A#a
				  DeclareMethod "p" @N/A#p

				@N/P
				  DeclareClass "N/P"
				  UseMethod @N/P#+

				@N/A
				  DeclareClass "N/A"
				  UseMethod @N/A#a
				  UseMethod @N/A#p

				Halt

				@N/P#+
				  LoadLocal 1
				  Return 2

				@N/A#a$lazy1
				  LoadLocal 0
				  CallMethod @N/A#p "test:" 7 46
				  ReturnLazy 1

				@N/A#a
				  LoadLocal 0
				  LoadLazy 1 @N/A#a$lazy1
				  LoadLocal 1
				  CallMethod @N/A#p "test:" 7 37
				  CallMethod @N/P#+ "test:" 7 37
				  Return 1

				@N/A#p
				  LoadObject @N/P
				  Return 1
            "#,
        );
    }

    #[test]
    fn refer_to_multiple_args_in_lazy() {
        assert_generates(
            Source::test(
                r#"
                namespace N.
                class P {
                    public + P p => p.
                }
                class A {
                    public a: P a b: P b c: P c =>
                        P + (a + b + c).
                }
                "#,
            ),
            r#"
            @N/P$methods
              DeclareMethod "+" @N/P#+

            @N/A$methods
              DeclareMethod "a:b:c:" @N/A#a:b:c:

            @N/P
              DeclareClass "N/P"
              UseMethod @N/P#+

            @N/A
              DeclareClass "N/A"
              UseMethod @N/A#a:b:c:

            Halt

            @N/P#+
              LoadLocal 1
              Return 2

            @N/A#a:b:c:$lazy1
              LoadLocal 0
              LoadLocal 2
              LoadLocal 4
              CallMethod @N/P#+ "test:" 8 30
              CallMethod @N/P#+ "test:" 8 29
              ReturnLazy 3

            @N/A#a:b:c:
              LoadLocal 1
              LoadLocal 3
              LoadLocal 5
              LoadLazy 3 @N/A#a:b:c:$lazy1
              LoadObject @N/P
              CallMethod @N/P#+ "test:" 8 25
              Return 4
            "#,
        );
    }
}
