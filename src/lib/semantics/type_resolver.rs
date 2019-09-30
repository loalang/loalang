use crate::semantics::*;
use crate::*;

pub struct TypeResolver {
    pub diagnostics: Vec<Diagnostic>,
    types: HashMap<*const Expression, Type>,
}

impl TypeResolver {
    pub fn new() -> TypeResolver {
        TypeResolver {
            diagnostics: vec![],
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
        match expression.as_ref() {
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
                                .insert(expression.as_ref() as *const _, mt.signature.return_type);
                            return;
                        }
                    }
                    self.diagnostics
                        .push(Diagnostic::MissingBehaviour(t.clone(), m.selector.clone()));
                    return;
                }
            }

            Expression::SelfExpression(_, class) => {
                self.types.insert(
                    expression.as_ref() as *const _,
                    Type {
                        constructor: TypeConstructor::SelfType(*class),
                        arguments: vec![], // TODO: Make types referencing all type params for the class.
                    },
                );
            }

            Expression::Reference(Reference::Unresolved(_)) => (),

            Expression::Reference(Reference::Class(c)) => {
                // TODO: Class constructor types
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
