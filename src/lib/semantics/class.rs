use crate::semantics::*;
use crate::*;

pub struct Class {
    pub name: Symbol,
    pub qualified_name: String,
    pub type_parameters: Vec<Arc<TypeParameter>>,
    pub super_types: Vec<Type>,
    pub variables: Vec<Arc<Variable>>,
    pub methods: Vec<Method>,
}

impl Class {
    pub fn callable_methods(&self) -> HashMap<Symbol, Method> {
        let mut methods = HashMap::<Symbol, Method>::new();

        for super_type in &self.super_types {
            methods.extend(super_type.callable_methods());
        }

        for own_method in &self.methods {
            methods.insert(own_method.selector().clone(), own_method.clone());
        }

        methods
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inherited_method() {
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
    fn overridden_method() {
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
