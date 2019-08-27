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
        Self::new_(None)
    }

    fn inner(outer: &'a LexicalScope<'a>) -> LexicalScope<'a> {
        Self::new_(Some(outer))
    }

    fn new_(parent: Option<&'a LexicalScope<'a>>) -> LexicalScope<'a> {
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

    pub fn register_class(&mut self, class: &Arc<Class>) {
        self.type_constructors
            .push(TypeConstructor::Class(class.clone()));
    }

    pub fn resolve_class(&self, mut class: Arc<Class>) -> Arc<Class> {
        {
            let class = Arc::get_mut(&mut class).unwrap();
            let mut class_scope = LexicalScope::inner(self);

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
                    .push(class_scope.resolve_type_parameter(param));
            }

            for super_type in std::mem::replace(&mut class.super_types, vec![]) {
                class.super_types.push(class_scope.resolve_type(super_type));
            }

            for variable in std::mem::replace(&mut class.variables, vec![]) {
                class.variables.push(class_scope.resolve_variable(variable));
            }

            for method in std::mem::replace(&mut class.methods, vec![]) {
                class.methods.push(class_scope.resolve_method(method));
            }
        }

        class
    }

    pub fn register_type_parameter(&mut self, _param: &Arc<TypeParameter>) {}

    pub fn resolve_type_parameter(&self, param: Arc<TypeParameter>) -> Arc<TypeParameter> {
        param
    }

    pub fn resolve_type(&self, typ: Type) -> Type {
        if let TypeConstructor::Unresolved(ref name) = typ.constructor {
            for available_constructor in self.type_constructors() {
                if available_constructor.name() == name {
                    return Type {
                        constructor: available_constructor.clone(),
                        arguments: typ
                            .arguments
                            .into_iter()
                            .map(|a| self.resolve_type(a))
                            .collect(),
                    };
                }
            }
        }
        typ
    }

    pub fn register_variable(&mut self, _variable: &Arc<Variable>) {}

    pub fn resolve_variable(&self, variable: Arc<Variable>) -> Arc<Variable> {
        variable
    }

    pub fn register_method(&mut self, _method: &Method) {}

    pub fn resolve_method(&self, method: Method) -> Method {
        method
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

        let typ = scope.resolve_type(typ);

        match typ.constructor {
            TypeConstructor::Class(ref c) if Arc::ptr_eq(&class, c) => (),
            _ => panic!("The resolved type did not receive the correct constructor"),
        }
    }
}
