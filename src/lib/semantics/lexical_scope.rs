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
    type_constructors: Vec<TypeConstructor>,
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
            type_constructors: vec![],
        }
    }

    fn type_constructors(&self) -> IterOrChain<TypeConstructor> {
        let iter = self.type_constructors.iter();

        if let Some(ref parent) = self.parent {
            return IterOrChain::Chain(iter.chain(Box::new(parent.type_constructors())));
        }

        IterOrChain::Iter(iter)
    }

    pub fn register_program(&mut self, program: &Program) {
        for class in program.classes.iter() {
            self.register_class(class);
        }
    }

    pub fn resolve_program(&self, mut program: Program) -> Diagnosed<Program> {
        Just(Program {
            classes: diagnose!(Diagnosed::extract_flat_map(
                std::mem::replace(&mut program.classes, vec![]),
                |class| self.resolve_class(class),
            )),
        })
    }

    pub fn register_class(&mut self, class: &Arc<Class>) {
        self.type_constructors
            .push(TypeConstructor::Class(&**class as *const Class));
    }

    pub fn resolve_class(&self, mut class: Arc<Class>) -> Diagnosed<Arc<Class>> {
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
                class
                    .type_parameters
                    .push(diagnose!(class_scope.resolve_type_parameter(param)));
            }

            for super_type in std::mem::replace(&mut class.super_types, vec![]) {
                class
                    .super_types
                    .push(diagnose!(class_scope.resolve_type(super_type)));
            }

            for variable in std::mem::replace(&mut class.variables, vec![]) {
                class
                    .variables
                    .push(diagnose!(class_scope.resolve_variable(variable)));
            }

            for method in std::mem::replace(&mut class.methods, vec![]) {
                class
                    .methods
                    .push(diagnose!(class_scope.resolve_method(method)));
            }
        }

        Just(class)
    }

    pub fn register_type_parameter(&mut self, param: &Arc<TypeParameter>) {
        self.type_constructors.push(TypeConstructor::TypeParameter(
            &**param as *const TypeParameter,
        ));
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
            for available_constructor in self.type_constructors() {
                if available_constructor.name() == name {
                    let mut t = Type {
                        constructor: available_constructor.clone(),
                        arguments: vec![],
                    };
                    for a in typ.arguments.into_iter() {
                        t.arguments.push(diagnose!(self.resolve_type(a)));
                    }
                    return Just(t);
                }
            }
            return Failure(vec![Diagnostic::UndefinedSymbol(name.clone())]);
        }
        Just(typ)
    }

    pub fn register_variable(&mut self, _variable: &Arc<Variable>) {}

    pub fn resolve_variable(&self, variable: Arc<Variable>) -> Diagnosed<Arc<Variable>> {
        Just(variable)
    }

    pub fn register_method(&mut self, _method: &Method) {}

    pub fn resolve_method(&self, method: Method) -> Diagnosed<Method> {
        Just(method)
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
}
