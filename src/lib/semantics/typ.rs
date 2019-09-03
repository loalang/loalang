use crate::semantics::*;
use crate::*;

#[derive(Clone, Debug)]
pub struct Type {
    pub constructor: TypeConstructor,
    pub arguments: Vec<Type>,
}

impl Type {
    pub fn callable_methods(&self) -> HashMap<Symbol, Method> {
        match self.constructor {
            TypeConstructor::Class(ref class) => unsafe { &**class }
                .callable_methods()
                .into_iter()
                .map(|(s, m)| {
                    (
                        s,
                        m.apply_type_arguments(
                            &self
                                .constructor
                                .type_parameters()
                                .iter()
                                .cloned()
                                .zip(self.arguments.iter().cloned())
                                .collect(),
                        ),
                    )
                })
                .collect(),

            TypeConstructor::TypeParameter(_) => HashMap::new(),

            TypeConstructor::Unresolved(Symbol(Some(ref span), ref s)) => panic!(
                "{} @ {}: Cannot get methods before resolving references",
                s, span
            ),
            TypeConstructor::Unresolved(Symbol(None, ref s)) => {
                panic!("{}: Cannot get methods before resolving references", s);
            }
            TypeConstructor::UnresolvedInteger => {
                panic!("Cannot get methods before resolving references")
            }
        }
    }

    pub fn apply_type_arguments(&self, arguments: &Vec<(Arc<TypeParameter>, Type)>) -> Type {
        if let TypeConstructor::TypeParameter(ref p) = self.constructor {
            for (pp, a) in arguments.iter() {
                if Arc::into_raw(pp.clone()) == *p {
                    return a.clone();
                }
            }
        }

        Type {
            constructor: self.constructor.clone(),
            arguments: self
                .arguments
                .iter()
                .map(|t| t.apply_type_arguments(arguments))
                .collect(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.constructor.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_type_arguments() {
        // List<a>
        let generic_class = class("List", |c| {
            let type_parameter = type_parameter(proper_type("Object"), "a", |_| {});
            c.type_parameters.push(type_parameter.clone());

            c.methods.push(partial_unary_method(
                "method",
                type_parameter_type(type_parameter),
            ));
        });

        // List<X>
        let mut generic_type = class_type(generic_class);
        generic_type.arguments.push(proper_type("X"));

        assert_eq!(
            format_methods(generic_type.callable_methods()),
            vec!["method -> X".to_string()]
        );
    }
}
