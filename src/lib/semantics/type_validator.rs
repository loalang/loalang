use crate::*;
use crate::semantics::*;

pub struct TypeValidator<'a> {
    types: &'a HashMap<*const Expression, Type>,
}

impl<'a> TypeValidator<'a> {
    pub fn new(types: &'a HashMap<*const Expression, Type>) -> TypeValidator<'a> {
        TypeValidator {
            types
        }
    }

    pub fn from_resolver(type_resolver: &'a TypeResolver) -> TypeValidator<'a> {
        Self::new(&type_resolver.types)
    }

    pub fn validate_program(&self, program: &Program) -> Diagnosed<()> {
        let mut diagnostics = vec![];
        for class in program.classes.iter() {
            diagnose!(diagnostics, self.validate_class(class));
        }
        Diagnosed::maybe_diagnosis((), diagnostics)
    }

    pub fn validate_class(&self, class: &Arc<Class>) -> Diagnosed<()> {
        let mut diagnostics = vec![];
        for method in class.methods.iter() {
            diagnose!(diagnostics, self.validate_method(method));
        }
        Diagnosed::maybe_diagnosis((), diagnostics)
    }

    pub fn validate_method(&self, method: &Method) -> Diagnosed<()> {
        match &method.implementation {
            Some(MethodImplementation::Body(_, expression)) => {
                self.check_assignable(self.type_of(expression), method.signature.return_type.clone())
            }
            Some(MethodImplementation::VariableGetter(_)) => {
                Just(())
            }
            Some(MethodImplementation::VariableSetter(_)) => {
                Just(())
            }
            None => Just(())
        }
    }

    fn type_of(&self, expression: &Arc<Expression>) -> Type {
        let id = expression.as_ref() as *const _;
        match self.types.get(&id) {
            Some(t) => t.clone(),

            #[cfg(test)]
            None => panic!("Couldn't get type of expression! Resolver messed up!"),

            #[cfg(not(test))]
            None => Type::unknown(),
        }
    }

    fn check_assignable(&self, span: Span, from: Type, to: Type) -> Diagnosed<()> {
        println!("Checking if {} is assignable to {}", from, to);
        if from.constructor == to.constructor {
            return Just(())
        }
        Diagnosis((), vec![
            Diagnostic::UnassignableType(span, from, to),
        ])
    }
}