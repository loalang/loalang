use crate::semantics::*;
use crate::*;

pub struct TypeResolver {
    types: HashMap<*const Expression, Type>,
}

impl TypeResolver {
    pub fn new() -> TypeResolver {
        TypeResolver {
            types: HashMap::new(),
        }
    }

    pub fn resolve_program(&mut self, program: &Program) {
        for class in program.classes.iter() {
            self.resolve_class(class);
        }
    }

    pub fn resolve_class(&mut self, class: &Arc<Class>) {
        for method in class.methods.iter() {
            self.resolve_method(method);
        }
    }

    pub fn resolve_method(&mut self, method: &Method) {
        if let Some(MethodImplementation::Body(_, ref body)) = method.implementation {
            self.resolve_expression(body);
        }
    }

    pub fn resolve_expression(&mut self, expression: &Arc<Expression>) {
        match &**expression {
            i @ Expression::Integer(_) => {
                self.types.insert(
                    i as *const Expression,
                    Type {
                        constructor: TypeConstructor::UnresolvedInteger,
                        arguments: vec![],
                    },
                );
            }

            Expression::MessageSend(r, m) => {
                self.resolve_expression(r);
                for a in m.arguments.iter() {
                    self.resolve_expression(a);
                }
                if let Some(t) = self.types.get(&(&**r as *const _)) {
                    for (s, mt) in t.callable_methods() {
                        if s == m.selector {
                            self.types
                                .insert(&**expression as *const _, mt.signature.return_type);
                            return;
                        }
                    }
                    panic!("`{}` doesn't respond to `{}`.", t, m.selector);
                }
            }

            Expression::Reference(Reference::Unresolved(_)) => (),

            Expression::Reference(Reference::Class(c)) => {
                self.types.insert(
                    &**expression as *const _,
                    Type {
                        constructor: TypeConstructor::Class(*c),
                        arguments: vec![],
                    },
                );
            }

            Expression::Reference(Reference::Binding(b)) => {
                let Binding(t, _) = unsafe { &**b };
                self.types.insert(&**expression as *const _, t.clone());
            }
        }
    }
}
