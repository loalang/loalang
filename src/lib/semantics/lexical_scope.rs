use crate::semantics::*;
use crate::*;

enum IterOrChain<'a, T> {
    Iter(Iter<'a, T>),
    Chain(std::iter::Chain<Iter<'a, T>, Box<IterOrChain<'a, T>>>),
}

impl<'a, T> Iterator for IterOrChain<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        match self {
            IterOrChain::Iter(ref mut i) => i.next(),
            IterOrChain::Chain(ref mut i) => i.next(),
        }
    }
}

pub struct LexicalScope<'a> {
    parent: Option<&'a LexicalScope<'a>>,
    classes: Vec<*const Class>,
    type_parameters: Vec<*const TypeParameter>,
    bindings: Vec<*const Binding>,
}

impl<'a> LexicalScope<'a> {
    pub fn new() -> LexicalScope<'a> {
        Self::with_parent(None)
    }

    fn inner(&'a self) -> LexicalScope<'a> {
        Self::with_parent(Some(self))
    }

    fn with_parent(parent: Option<&'a LexicalScope<'a>>) -> LexicalScope<'a> {
        LexicalScope {
            parent,
            classes: vec![],
            type_parameters: vec![],
            bindings: vec![],
        }
    }

    fn classes(&self) -> IterOrChain<*const Class> {
        let iter = self.classes.iter();

        if let Some(ref parent) = self.parent {
            return IterOrChain::Chain(iter.chain(Box::new(parent.classes())));
        }

        IterOrChain::Iter(iter)
    }

    fn type_parameters(&self) -> IterOrChain<*const TypeParameter> {
        let iter = self.type_parameters.iter();

        if let Some(ref parent) = self.parent {
            return IterOrChain::Chain(iter.chain(Box::new(parent.type_parameters())));
        }

        IterOrChain::Iter(iter)
    }

    fn bindings(&self) -> IterOrChain<*const Binding> {
        let iter = self.bindings.iter();

        if let Some(ref parent) = self.parent {
            return IterOrChain::Chain(iter.chain(Box::new(parent.bindings())));
        }

        IterOrChain::Iter(iter)
    }

    fn type_constructors(&self) -> Vec<TypeConstructor> {
        let mut o = vec![];
        for c in self.classes() {
            o.push(TypeConstructor::Class(*c))
        }
        for t in self.type_parameters() {
            o.push(TypeConstructor::TypeParameter(*t))
        }
        o
    }

    pub fn register_program(&mut self, program: &Program) {
        for class in program.classes.iter() {
            self.register_class(class);
        }
    }

    pub fn resolve_program(&self, mut program: Program) -> Diagnosed<Program> {
        let mut diagnostics = vec![];
        let program = Program {
            classes: diagnose!(
                diagnostics,
                Diagnosed::extract_flat_map(
                    std::mem::replace(&mut program.classes, vec![]),
                    |class| self.resolve_class(class),
                )
            ),
        };
        Diagnosed::maybe_diagnosis(program, diagnostics)
    }

    pub fn register_class(&mut self, class: &Arc<Class>) {
        self.classes.push(&**class as *const Class);
    }

    pub fn resolve_class(&self, mut class: Arc<Class>) -> Diagnosed<Arc<Class>> {
        let mut diagnostics = vec![];
        {
            let class = Arc::get_mut(&mut class).unwrap();
            let mut class_scope = self.inner();

            for param in class.type_parameters.iter() {
                class_scope.register_type_parameter(param);
            }

            for variable in class.variables.iter() {
                class_scope.register_variable(variable);
            }

            for method in class.methods.iter() {
                class_scope.register_method(method);
            }

            for param in std::mem::replace(&mut class.type_parameters, vec![]) {
                class.type_parameters.push(diagnose!(
                    diagnostics,
                    class_scope.resolve_type_parameter(param)
                ));
            }

            for super_type in std::mem::replace(&mut class.super_types, vec![]) {
                class
                    .super_types
                    .push(diagnose!(diagnostics, class_scope.resolve_type(super_type)));
            }

            for variable in std::mem::replace(&mut class.variables, vec![]) {
                class.variables.push(diagnose!(
                    diagnostics,
                    class_scope.resolve_variable(variable)
                ));
            }

            for method in std::mem::replace(&mut class.methods, vec![]) {
                class
                    .methods
                    .push(diagnose!(diagnostics, class_scope.resolve_method(method)));
            }
        }

        Diagnosed::maybe_diagnosis(class, diagnostics)
    }

    pub fn register_type_parameter(&mut self, param: &Arc<TypeParameter>) {
        self.type_parameters.push(&**param as *const TypeParameter);
    }

    pub fn resolve_type_parameter(
        &self,
        mut param: Arc<TypeParameter>,
    ) -> Diagnosed<Arc<TypeParameter>> {
        {
            let param = Arc::get_mut(&mut param).unwrap();

            param.constraint = diagnose!(self.resolve_type(param.constraint.clone()));
            for hk in std::mem::replace(&mut param.type_parameters, vec![]) {
                param
                    .type_parameters
                    .push(diagnose!(self.resolve_type_parameter(hk)));
            }
        }
        Just(param)
    }

    pub fn resolve_type(&self, typ: Type) -> Diagnosed<Type> {
        if let TypeConstructor::Unresolved(ref name) = typ.constructor {
            let mut t = Type {
                constructor: TypeConstructor::Unresolved(name.clone()),
                arguments: vec![],
            };
            let mut defined = false;
            for available_constructor in self.type_constructors() {
                if available_constructor.name() == name {
                    t = Type {
                        constructor: available_constructor.clone(),
                        arguments: vec![],
                    };
                    defined = true;
                }
            }
            for a in typ.arguments.into_iter() {
                t.arguments.push(diagnose!(self.resolve_type(a)));
            }
            if defined {
                return Just(t);
            } else {
                return Diagnosis(t, vec![Diagnostic::UndefinedSymbol(name.clone())]);
            }
        }
        Just(typ)
    }

    pub fn register_variable(&mut self, _variable: &Arc<Variable>) {}

    pub fn resolve_variable(&self, variable: Arc<Variable>) -> Diagnosed<Arc<Variable>> {
        Just(variable)
    }

    pub fn register_method(&mut self, method: &Method) {
        self.register_signature(&method.signature);
        if let Some(ref implementation) = method.implementation {
            self.register_method_implementation(implementation);
        }
    }

    pub fn resolve_method(&self, mut method: Method) -> Diagnosed<Method> {
        let mut diagnostics = vec![];
        method.signature = diagnose!(diagnostics, self.resolve_signature(method.signature));
        if let Some(implementation) = method.implementation {
            method.implementation = Some(diagnose!(
                diagnostics,
                self.resolve_method_implementation(implementation)
            ));
        }
        Diagnosed::maybe_diagnosis(method, diagnostics)
    }

    pub fn register_signature(&mut self, signature: &Signature) {
        for type_parameter in signature.type_parameters.iter() {
            self.register_type_parameter(type_parameter);
        }
    }

    pub fn resolve_signature(&self, mut signature: Signature) -> Diagnosed<Signature> {
        let mut diagnostics = vec![];

        for type_parameter in std::mem::replace(&mut signature.type_parameters, vec![]) {
            signature.type_parameters.push(diagnose!(
                diagnostics,
                self.resolve_type_parameter(type_parameter)
            ));
        }

        for parameter in std::mem::replace(&mut signature.parameters, vec![]) {
            signature
                .parameters
                .push(diagnose!(diagnostics, self.resolve_type(parameter)));
        }

        signature.return_type = diagnose!(diagnostics, self.resolve_type(signature.return_type));

        Diagnosed::maybe_diagnosis(signature, diagnostics)
    }

    pub fn register_method_implementation(&mut self, _implementation: &MethodImplementation) {}

    pub fn resolve_method_implementation(
        &self,
        implementation: MethodImplementation,
    ) -> Diagnosed<MethodImplementation> {
        match implementation {
            MethodImplementation::Body(patterns, body) => {
                let mut diagnostics = vec![];
                let mut method_scope = self.inner();
                for pattern in patterns.iter() {
                    method_scope.register_pattern(pattern);
                }
                method_scope.register_expression(&body);

                let body = MethodImplementation::Body(
                    diagnose!(
                        diagnostics,
                        Diagnosed::extract_flat_map(patterns, |pattern| {
                            method_scope.resolve_pattern(pattern)
                        })
                    ),
                    diagnose!(diagnostics, method_scope.resolve_expression(body)),
                );
                Diagnosed::maybe_diagnosis(body, diagnostics)
            }
            m => Just(m),
        }
    }

    pub fn register_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Binding(binding) => self.register_binding(binding),
        }
    }

    pub fn resolve_pattern(&self, pattern: Pattern) -> Diagnosed<Pattern> {
        Just(match pattern {
            Pattern::Binding(binding) => Pattern::Binding(diagnose!(self.resolve_binding(binding))),
        })
    }

    pub fn register_binding(&mut self, binding: &Arc<Binding>) {
        self.bindings.push(&**binding as *const _);
    }

    pub fn resolve_binding(&self, mut binding: Arc<Binding>) -> Diagnosed<Arc<Binding>> {
        {
            let mut binding = Arc::get_mut(&mut binding).unwrap();
            binding.0 = diagnose!(self.resolve_type(binding.0.clone()));
        }
        Just(binding)
    }

    pub fn register_expression(&mut self, _expression: &Arc<Expression>) {}

    pub fn resolve_expression(&self, expression: Arc<Expression>) -> Diagnosed<Arc<Expression>> {
        let mut diagnostics = vec![];
        let exp = match &*expression {
            Expression::Integer(_) => expression,
            Expression::MessageSend(rcv, message) => Arc::new(Expression::MessageSend(
                diagnose!(diagnostics, self.resolve_expression(rcv.clone())),
                diagnose!(diagnostics, self.resolve_message(message.clone())),
            )),
            Expression::Reference(reference) => Arc::new(Expression::Reference(diagnose!(
                diagnostics,
                self.resolve_reference(reference.clone())
            ))),
            Expression::SelfExpression(_, _) => expression,
        };
        Diagnosed::maybe_diagnosis(exp, diagnostics)
    }

    pub fn resolve_message(&self, mut message: Message) -> Diagnosed<Message> {
        message.arguments = diagnose!(Diagnosed::extract_flat_map(message.arguments, |arg| {
            self.resolve_expression(arg)
        }));
        Just(message)
    }

    pub fn resolve_reference(&self, reference: Reference) -> Diagnosed<Reference> {
        match reference {
            Reference::Unresolved(s) => {
                for binding in self.bindings() {
                    let Binding(_, ss) = unsafe { &**binding };
                    if *ss == s {
                        return Just(Reference::Binding(*binding));
                    }
                }
                return Diagnosis(
                    Reference::Unresolved(s.clone()),
                    vec![Diagnostic::UndefinedSymbol(s)],
                );
            }
            r => Just(r),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn referencing_a_class() {
        let class = class("X", |_| {});
        let typ = Type {
            constructor: TypeConstructor::Unresolved(symbol("X")),
            arguments: vec![],
        };

        let mut scope = LexicalScope::new();

        scope.register_class(&class);

        let typ = scope.resolve_type(typ).unwrap();

        match typ.constructor {
            TypeConstructor::Class(ref c) if Arc::into_raw(class) == *c => (),
            _ => panic!("The resolved type did not receive the correct constructor"),
        }
    }

    #[test]
    fn circular_reference() {
        let type_x = Type {
            constructor: TypeConstructor::Unresolved(symbol("X")),
            arguments: vec![],
        };
        let type_y = Type {
            constructor: TypeConstructor::Unresolved(symbol("Y")),
            arguments: vec![],
        };

        let class_x = class("X", |c| c.super_types.push(type_y));
        let class_y = class("Y", |c| c.super_types.push(type_x));

        let mut scope = LexicalScope::new();
        scope.register_class(&class_x);
        scope.register_class(&class_y);

        let class_x = scope.resolve_class(class_x).unwrap();
        let class_y = scope.resolve_class(class_y).unwrap();

        match class_x.super_types[0].constructor {
            TypeConstructor::Class(ref c) if &*class_y as *const Class == *c => (),
            _ => panic!("class X's super type was not resolved to Y correctly"),
        }
        match class_y.super_types[0].constructor {
            TypeConstructor::Class(ref c) if &*class_x as *const Class == *c => (),
            _ => panic!("class Y's super type was not resolved to X correctly"),
        }
    }

    #[test]
    fn integration() {
        let mut parser = syntax::Parser::new(&Source::test(
            r#"
            class X<Y y> {
              public <y y'> y -> X => 123.
            }
            class Y<X x>.
            "#,
        ));
        let module = parser.parse_module().unwrap();
        let program = Resolver::new().resolve_modules(&vec![module]);
        let mut global_scope = LexicalScope::new();
        global_scope.register_program(&program);
        let program = global_scope.resolve_program(program).unwrap();

        let class_x = &program.classes[0];
        let class_y = &program.classes[1];

        let type_parameter_y = &class_x.type_parameters[0];

        assert_eq!(program.classes.len(), 2);
        match type_parameter_y.constraint.constructor {
            TypeConstructor::Class(c) => assert_eq!(unsafe { &*c }.name.to_string(), "Y"),
            _ => panic!("Expected a class type constructor"),
        }
        match class_y.type_parameters[0].constraint.constructor {
            TypeConstructor::Class(c) => assert_eq!(unsafe { &*c }.name.to_string(), "X"),
            _ => panic!("Expected a class type constructor"),
        }

        match class_x
            .callable_methods()
            .get(&symbol("y"))
            .unwrap()
            .signature
            .type_parameters[0]
            .constraint
            .constructor
        {
            TypeConstructor::TypeParameter(p) => {
                assert_eq!(p, &**type_parameter_y as *const TypeParameter)
            }
            _ => panic!("Expected a type parameter type constructor"),
        }
    }
}
