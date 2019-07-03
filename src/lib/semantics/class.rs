use crate::semantics::*;
use crate::*;

pub struct Class {
    pub name: Symbol,
    pub type_parameters: Vec<Arc<TypeParameter>>,
    pub super_types: Vec<Type>,
    pub variables: Vec<Arc<Variable>>,
    pub methods: Vec<Method>,
}

impl Class {
    pub fn callable_methods(&self) -> HashMap<Symbol, &Method> {
        let mut methods = HashMap::<Symbol, &Method>::new();

        for super_type in &self.super_types {
            methods.extend(super_type.callable_methods());
        }

        for own_method in &self.methods {
            methods.insert(own_method.selector().clone(), own_method);
        }

        methods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class<F: FnOnce(&mut Class)>(name: &str, f: F) -> Arc<Class> {
        let mut class = Class {
            name: Symbol(name.into()),
            type_parameters: vec![],
            super_types: vec![],
            variables: vec![],
            methods: vec![],
        };
        f(&mut class);
        Arc::new(class)
    }

    fn class_type(class: Arc<Class>) -> Type {
        Type {
            constructor: TypeConstructor::Class(class),
            arguments: vec![],
        }
    }

    fn proper_type(name: &str) -> Type {
        class_type(class(name, |_| {}))
    }

    fn partial_unary_method(selector: &str, return_type: Type) -> Method {
        Method {
            signature: Signature {
                selector: Symbol(selector.into()),
                type_parameters: vec![],
                parameters: vec![],
                return_type,
            },
            implementation: None,
        }
    }

    fn format_methods(methods: HashMap<Symbol, &Method>) -> Vec<String> {
        let mut methods = methods
            .iter()
            .map(|(_, m)| format!("{}", m.signature))
            .collect::<Vec<_>>();

        methods.sort();

        methods
    }

    #[test]
    fn methods_of_sub_class() {
        let x_type = proper_type("X");

        let method_a = partial_unary_method("a", x_type.clone());
        let method_b = partial_unary_method("b", x_type.clone());

        let super_class = class("A", |c| {
            c.methods.push(method_a);
        });

        let sub_class = class("B", |c| {
            c.super_types.push(class_type(super_class));
            c.methods.push(method_b);
        });

        assert_eq!(
            format_methods(sub_class.callable_methods()),
            vec!["a -> X".to_string(), "b -> X".to_string()]
        );
    }

    #[test]
    fn overridden_methods() {
        let x_type = proper_type("X");
        let y_type = proper_type("Y");

        let method_a1 = partial_unary_method("a", x_type);
        let method_a2 = partial_unary_method("a", y_type);

        let super_class = class("A", |c| {
            c.methods.push(method_a1);
        });

        let sub_class = class("B", |c| {
            c.super_types.push(class_type(super_class));
            c.methods.push(method_a2);
        });

        assert_eq!(
            format_methods(sub_class.callable_methods()),
            vec!["a -> Y".to_string()]
        );
    }
}
