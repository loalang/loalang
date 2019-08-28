use crate::*;

pub struct Resolver;

impl Resolver {
    pub fn new() -> Resolver {
        Resolver
    }

    pub fn resolve_modules(&mut self, modules: &Vec<syntax::Module>) -> semantics::Program {
        let mut program = semantics::Program { classes: vec![] };

        for syntax::Module(classes) in modules.iter() {
            for class in classes.iter() {
                program.classes.push(self.resolve_class(class));
            }
        }

        program
    }

    pub fn resolve_expression(&mut self, cst: &syntax::Expression) -> Arc<semantics::Expression> {
        Arc::new(match cst {
            syntax::Expression::Integer(syntax::Integer(syntax::Token { kind, .. })) => {
                match kind {
                    syntax::TokenKind::SimpleInteger(i) => semantics::Expression::Integer(
                        BigInt::parse_bytes(i.as_bytes(), 10).unwrap(),
                    ),
                    _ => semantics::Expression::Integer(BigInt::new(
                        num_bigint::Sign::NoSign,
                        vec![0],
                    )),
                }
            }

            syntax::Expression::MessageSend(box syntax::MessageSend::Unary(receiver, id)) => {
                let message = semantics::Message {
                    selector: self.resolve_identifier(id),
                    arguments: vec![],
                };
                semantics::Expression::MessageSend(self.resolve_expression(receiver), message)
            }
            syntax::Expression::MessageSend(box syntax::MessageSend::Binary(receiver, op, arg)) => {
                let message = semantics::Message {
                    selector: semantics::Symbol(Some(op.span.clone()), op.lexeme()),
                    arguments: vec![self.resolve_expression(arg)],
                };
                semantics::Expression::MessageSend(self.resolve_expression(receiver), message)
            }
            syntax::Expression::MessageSend(box syntax::MessageSend::Keyword(receiver, args)) => {
                let message = semantics::Message {
                    selector: semantics::Symbol(
                        Some(cst.span()),
                        args.iter()
                            .map(|(k, _)| (k as &dyn format::Format).to_string())
                            .collect(),
                    ),
                    arguments: args
                        .iter()
                        .map(|(_, a)| self.resolve_expression(a))
                        .collect(),
                };
                semantics::Expression::MessageSend(self.resolve_expression(receiver), message)
            }
        })
    }

    pub fn resolve_identifier(
        &mut self,
        syntax::Identifier(t): &syntax::Identifier,
    ) -> semantics::Symbol {
        semantics::Symbol(Some(t.span.clone()), t.lexeme())
    }

    pub fn resolve_method(
        &mut self,
        visibility: semantics::Visibility,
        method: &syntax::Method,
    ) -> semantics::Method {
        match method {
            syntax::Method::Concrete(syntax::ConcreteMethod(
                type_parameters,
                message_pattern,
                return_type,
                syntax::MethodBody(_, expression),
            )) => {
                let type_parameters = match type_parameters {
                    None => vec![],
                    Some(syntax::TypeParameterList(_, tps, _)) => tps
                        .iter()
                        .map(|tp| self.resolve_type_parameter(tp))
                        .collect(),
                };
                let syntax::ReturnType(_, return_type) =
                    return_type.as_ref().expect("TODO: INFER RETURN TYPE");
                let return_type = self.resolve_type(&return_type);

                let signature;
                let parameters;
                match message_pattern {
                    syntax::MessagePattern::Unary(id) => {
                        signature = semantics::Signature {
                            type_parameters,
                            selector: self.resolve_identifier(id),
                            parameters: vec![],
                            return_type,
                        };
                        parameters = vec![];
                    }

                    syntax::MessagePattern::Binary(op, param) => {
                        let param = self.resolve_pattern(param);
                        signature = semantics::Signature {
                            type_parameters,
                            selector: semantics::Symbol(Some(op.span.clone()), op.lexeme()),
                            parameters: vec![param.typ()],
                            return_type,
                        };
                        parameters = vec![param];
                    }

                    syntax::MessagePattern::Keyword(kws) => {
                        let mut params = vec![];
                        let mut selector = String::new();
                        for (kw, p) in kws.iter() {
                            selector.push_str((kw as &dyn format::Format).to_string().as_ref());
                            params.push(self.resolve_pattern(p));
                        }
                        signature = semantics::Signature {
                            type_parameters,
                            selector: semantics::Symbol(Some(message_pattern.span()), selector),
                            parameters: params.iter().map(|p| p.typ()).collect(),
                            return_type,
                        };
                        parameters = params;
                    }
                }

                semantics::Method {
                    visibility,
                    signature,
                    implementation: Some(semantics::MethodImplementation::Body(
                        parameters,
                        self.resolve_expression(expression),
                    )),
                }
            }
        }
    }

    pub fn resolve_pattern(&mut self, pattern: &syntax::Pattern) -> semantics::Pattern {
        match pattern {
            syntax::Pattern::Binding(t, i) => semantics::Pattern::Binding(
                self.resolve_type(t.as_ref().expect("TODO: INFER BINDING TYPE")),
                self.resolve_identifier(i),
            ),
        }
    }

    pub fn resolve_type(&mut self, typ: &syntax::Type) -> semantics::Type {
        match typ {
            syntax::Type::Class(id, args) => semantics::Type {
                constructor: semantics::TypeConstructor::Unresolved(self.resolve_identifier(id)),
                arguments: match args {
                    None => vec![],
                    Some(syntax::TypeArgumentList(_, a, _)) => {
                        a.iter().map(|a| self.resolve_type(a)).collect()
                    }
                },
            },
        }
    }

    pub fn resolve_type_parameter(
        &mut self,
        syntax::TypeParameter(t, id, v): &syntax::TypeParameter,
    ) -> Arc<semantics::TypeParameter> {
        Arc::new(semantics::TypeParameter {
            constraint: self
                .resolve_type(t.as_ref().expect("TODO: DEFAULT TYPE PARAMETER CONSTRAINT")),
            name: self.resolve_identifier(id),
            type_parameters: vec![],
            variance: match v {
                None | Some(syntax::Variance::Inout(_)) => semantics::Variance::Invariant,
                Some(syntax::Variance::In(_)) => semantics::Variance::Contravariant,
                Some(syntax::Variance::Out(_)) => semantics::Variance::Covariant,
            },
        })
    }

    pub fn resolve_class(
        &mut self,
        syntax::Class(_, id, tp, body): &syntax::Class,
    ) -> Arc<semantics::Class> {
        let mut class = semantics::Class {
            name: self.resolve_identifier(id),
            type_parameters: tp
                .as_ref()
                .map(|syntax::TypeParameterList(_, tps, _)| {
                    tps.iter()
                        .map(|tp| self.resolve_type_parameter(tp))
                        .collect()
                })
                .unwrap_or(vec![]),

            // Class members
            super_types: vec![],
            variables: vec![],
            methods: vec![],
        };

        if let syntax::ClassBody::Braced(_, members, _) = body {
            for member in members.iter() {
                match member {
                    syntax::ClassMember::Method(v, m, _) => {
                        let v = self.resolve_visibility(v);
                        class.methods.push(self.resolve_method(v, m));
                    }
                }
            }
        }

        Arc::new(class)
    }

    pub fn resolve_visibility(&mut self, token: &syntax::Token) -> semantics::Visibility {
        match token.kind {
            syntax::TokenKind::PublicKeyword => semantics::Visibility::Public,
            syntax::TokenKind::PrivateKeyword => semantics::Visibility::Private,
            _ => panic!("Invalid visibility keyword"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parses_resolves_and_formats_method(s: &str) {
        let method = Resolver::new().resolve_method(
            semantics::Visibility::Public,
            &syntax::Parser::new(&Source::test(s))
                .parse_method()
                .unwrap(),
        );
        (&method as &dyn format::Format).to_string();
    }

    fn assert_parses_resolves_and_formats_class(s: &str) {
        let method = Resolver::new()
            .resolve_class(&syntax::Parser::new(&Source::test(s)).parse_class().unwrap());
        (&method as &dyn format::Format).to_string();
    }

    #[test]
    fn minimal_method() {
        assert_parses_resolves_and_formats_method("x -> Integer => 12");
    }

    #[test]
    fn maximal_method() {
        assert_parses_resolves_and_formats_method(
            "<List<X> a in, Object b out> method: a x with: a y -> Dictionary<X, Y> => 12 + 42",
        );
    }

    #[test]
    fn empty_class() {
        assert_parses_resolves_and_formats_class("class Empty.");
    }

    #[test]
    fn complex_class_with_methods() {
        assert_parses_resolves_and_formats_class(
            r#"
                class X<Other<Type> x in, Object y inout, Object z> {
                  public <Object u> method: u u' with: u u'' -> U<u> =>
                    123 add: 123.
                }
            "#,
        );
    }
}
