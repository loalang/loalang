use crate::syntax::*;
use crate::*;
use TokenKind::*;

macro_rules! consume {
    ($self: expr, $($kind: tt)+) => {{
        let t = $self.tokens[$self.offset].clone();
        if !matches!(t.kind, $($kind)+) {
            return Failure(vec![Diagnostic::UnexpectedToken(
                t,
                stringify!($($kind)+).into(),
            )]);
        }
        $self.offset += 1;
        t
    }};
}

macro_rules! sees {
    ($ahead: expr => $self: expr, $($kind: tt)+) => {{
        let t = &$self.tokens[$self.offset + $ahead];
        matches!(t.kind, $($kind)+)
    }};

    ($self: expr, $($kind: tt)+) => {{
        sees!(0 => $self, $($kind)+)
    }};
}

pub struct Parser {
    offset: usize,
    tokens: Arc<Vec<Token>>,
}

impl Parser {
    pub fn new(source: &Arc<Source>) -> Parser {
        Parser {
            offset: 0,
            tokens: Arc::new(
                tokenize(source.clone())
                    .into_iter()
                    .filter(|t| !matches!(t.kind, LineComment(_) | Whitespace(_)))
                    .collect(),
            ),
        }
    }

    pub fn parse_module(&mut self) -> Diagnosed<Module> {
        let mut classes = vec![];
        while !sees!(self, EOF) {
            classes.push(diagnose!(self.parse_class()));
        }
        Just(Module(classes))
    }

    pub fn parse_integer(&mut self) -> Diagnosed<Integer> {
        Just(Integer(consume!(self, SimpleInteger(_))))
    }

    pub fn parse_expression(&mut self) -> Diagnosed<Expression> {
        let e = diagnose!(self.parse_leaf_expression());
        let e = diagnose!(self.parse_potential_unary(e));
        let e = diagnose!(self.parse_potential_binary(e));
        let e = diagnose!(self.parse_potential_keyword(e));
        Just(e)
    }

    fn parse_leaf_expression(&mut self) -> Diagnosed<Expression> {
        Just(Expression::Integer(diagnose!(self.parse_integer())))
    }

    fn parse_potential_unary(&mut self, receiver: Expression) -> Diagnosed<Expression> {
        if sees!(self, SimpleSymbol(_)) && !sees!(1 => self, Colon) {
            let message = diagnose!(self.parse_identifier());

            let e = MessageSend::Unary(receiver, message);
            let e = Expression::MessageSend(Box::new(e));
            let e = diagnose!(self.parse_potential_unary(e));
            Just(e)
        } else {
            Just(receiver)
        }
    }

    fn parse_potential_binary(&mut self, receiver: Expression) -> Diagnosed<Expression> {
        if sees!(self, Plus) {
            let message = consume!(self, Plus);

            let operand = diagnose!(self.parse_leaf_expression());
            let operand = diagnose!(self.parse_potential_unary(operand));

            let e = MessageSend::Binary(receiver, message, operand);
            let e = Expression::MessageSend(Box::new(e));
            let e = diagnose!(self.parse_potential_unary(e));
            let e = diagnose!(self.parse_potential_binary(e));
            Just(e)
        } else {
            Just(receiver)
        }
    }

    fn parse_potential_keyword(&mut self, receiver: Expression) -> Diagnosed<Expression> {
        if sees!(self, SimpleSymbol(_)) {
            let mut keywords = vec![];

            while sees!(self, SimpleSymbol(_)) {
                let keyword = diagnose!(self.parse_keyword());

                let a = diagnose!(self.parse_leaf_expression());
                let a = diagnose!(self.parse_potential_unary(a));
                let a = diagnose!(self.parse_potential_binary(a));
                keywords.push((keyword, a));
            }

            let e = MessageSend::Keyword(receiver, keywords.into());
            let e = Expression::MessageSend(Box::new(e));
            let e = diagnose!(self.parse_potential_unary(e));
            let e = diagnose!(self.parse_potential_binary(e));
            let e = diagnose!(self.parse_potential_keyword(e));
            Just(e)
        } else {
            Just(receiver)
        }
    }

    pub fn parse_keyword(&mut self) -> Diagnosed<Keyword> {
        Just(Keyword(
            diagnose!(self.parse_identifier()),
            consume!(self, Colon),
        ))
    }

    pub fn parse_identifier(&mut self) -> Diagnosed<Identifier> {
        Just(Identifier(consume!(self, SimpleSymbol(_))))
    }

    pub fn parse_method(&mut self) -> Diagnosed<Method> {
        Just(Method::Concrete(diagnose!(self.parse_concrete_method())))
    }

    pub fn parse_concrete_method(&mut self) -> Diagnosed<ConcreteMethod> {
        Just(ConcreteMethod(
            if sees!(self, OpenAngle) {
                Some(diagnose!(self.parse_type_parameter_list()))
            } else {
                None
            },
            diagnose!(self.parse_message_pattern()),
            if sees!(self, Arrow) {
                Some(diagnose!(self.parse_return_type()))
            } else {
                None
            },
            diagnose!(self.parse_method_body()),
        ))
    }

    pub fn parse_type_parameter_list(&mut self) -> Diagnosed<TypeParameterList> {
        Just(TypeParameterList(
            consume!(self, OpenAngle),
            {
                let mut params = vec![];
                while !sees!(self, CloseAngle) {
                    params.push(diagnose!(self.parse_type_parameter()));
                    if !sees!(self, Comma) {
                        break;
                    }
                    consume!(self, Comma);
                }
                params
            },
            consume!(self, CloseAngle),
        ))
    }

    pub fn parse_type_parameter(&mut self) -> Diagnosed<TypeParameter> {
        Just(TypeParameter(
            if !sees!(1 => self, InKeyword | OutKeyword | InoutKeyword | CloseAngle) {
                Some(diagnose!(self.parse_type()))
            } else {
                None
            },
            diagnose!(self.parse_identifier()),
            if !sees!(self, CloseAngle) {
                Some(diagnose!(self.parse_variance()))
            } else {
                None
            },
        ))
    }

    pub fn parse_variance(&mut self) -> Diagnosed<Variance> {
        let token = consume!(self, InKeyword | OutKeyword | InoutKeyword);

        Just(match token.kind {
            InKeyword => Variance::In(token),
            OutKeyword => Variance::Out(token),
            InoutKeyword => Variance::Inout(token),
            _ => panic!("Invalid state"),
        })
    }

    pub fn parse_type(&mut self) -> Diagnosed<Type> {
        Just(Type::Class(
            diagnose!(self.parse_identifier()),
            if sees!(self, OpenAngle) {
                Some(diagnose!(self.parse_type_argument_list()))
            } else {
                None
            },
        ))
    }

    pub fn parse_type_argument_list(&mut self) -> Diagnosed<TypeArgumentList> {
        Just(TypeArgumentList(
            consume!(self, OpenAngle),
            {
                let mut args = vec![];
                while !sees!(self, CloseAngle) {
                    args.push(diagnose!(self.parse_type()));
                    if !sees!(self, Comma) {
                        break;
                    }
                    consume!(self, Comma);
                }
                args
            },
            consume!(self, CloseAngle),
        ))
    }

    pub fn parse_message_pattern(&mut self) -> Diagnosed<MessagePattern> {
        Just(if sees!(self, SimpleSymbol(_)) && sees!(1 => self, Colon) {
            MessagePattern::Keyword({
                let mut keywords = vec![];
                while sees!(self, SimpleSymbol(_)) {
                    keywords.push((
                        diagnose!(self.parse_keyword()),
                        diagnose!(self.parse_pattern()),
                    ));
                }
                keywords.into()
            })
        } else if sees!(self, SimpleSymbol(_)) {
            MessagePattern::Unary(diagnose!(self.parse_identifier()))
        } else {
            MessagePattern::Binary(consume!(self, Plus), diagnose!(self.parse_pattern()))
        })
    }

    pub fn parse_pattern(&mut self) -> Diagnosed<Pattern> {
        Just(Pattern::Binding(
            if sees!(1 => self, OpenAngle) || !sees!(2 => self, Colon) {
                Some(diagnose!(self.parse_type()))
            } else {
                None
            },
            diagnose!(self.parse_identifier()),
        ))
    }

    pub fn parse_return_type(&mut self) -> Diagnosed<ReturnType> {
        Just(ReturnType(
            consume!(self, Arrow),
            diagnose!(self.parse_type()),
        ))
    }

    pub fn parse_method_body(&mut self) -> Diagnosed<MethodBody> {
        Just(MethodBody(
            consume!(self, FatArrow),
            diagnose!(self.parse_expression()),
        ))
    }

    pub fn parse_class(&mut self) -> Diagnosed<Class> {
        Just(Class(
            consume!(self, ClassKeyword),
            diagnose!(self.parse_identifier()),
            if sees!(self, OpenAngle) {
                Some(diagnose!(self.parse_type_parameter_list()))
            } else {
                None
            },
            diagnose!(self.parse_class_body()),
        ))
    }

    pub fn parse_class_body(&mut self) -> Diagnosed<ClassBody> {
        if sees!(self, Period) {
            Just(ClassBody::Empty(consume!(self, Period)))
        } else {
            Just(ClassBody::Braced(
                consume!(self, OpenCurly),
                {
                    let mut members = vec![];
                    while !sees!(self, CloseCurly) {
                        members.push(diagnose!(self.parse_class_member()));
                    }
                    members
                },
                consume!(self, CloseCurly),
            ))
        }
    }

    pub fn parse_class_member(&mut self) -> Diagnosed<ClassMember> {
        Just(ClassMember::Method(
            consume!(self, PrivateKeyword | PublicKeyword),
            diagnose!(self.parse_method()),
            consume!(self, Period),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_parses {
        ($name: ident, $code: expr, $method: ident, $($pattern:tt)+) => {
            #[test]
            fn $name() {
                let mut parser = Parser::new(&Source::test($code));
                let c = assert_diagnose!(parser.$method());
                assert_matches!(c, $($pattern)+);
            }
        };
    }

    #[test]
    fn simple_integer() {
        let mut parser = Parser::new(&Source::test("12"));
        let Integer(t) = assert_diagnose!(parser.parse_integer());
        assert_matches!(t.kind, SimpleInteger(ref s) if s == "12");
    }

    assert_parses! {
        unary_message_send,
        "12 negated",
        parse_expression,
        Expression::MessageSend(
            box MessageSend::Unary(
                Expression::Integer(
                    Integer(Token {
                        kind: SimpleInteger(ref s),
                        ..
                    })
                ),
                Identifier(Token {
                    kind: SimpleSymbol(ref m),
                    ..
                })
            )
        ) if s == "12" && m == "negated"
    }

    assert_parses! {
        binary_message_send,
        "12 + 2",
        parse_expression,
        Expression::MessageSend(
            box MessageSend::Binary(Expression::Integer(_), Token { kind: Plus, .. }, Expression::Integer(_))
        )
    }

    assert_parses! {
        keyword_message_send,
        "12 a: 2 b: 3",
        parse_expression,
        Expression::MessageSend(
            box MessageSend::Keyword(
                Expression::Integer(_),
                box [
                    (Keyword(Identifier(Token { kind: SimpleSymbol(ref k1), .. }), _), Expression::Integer(_)),
                    (Keyword(Identifier(Token { kind: SimpleSymbol(ref k2), .. }), _), Expression::Integer(_)),
                ]
            )
        ) if k1 == "a" && k2 == "b"
    }

    assert_parses! {
        complex_expression,
        "13 x + 42 x: 2 + 2 x + 2 x: 5",
        parse_expression,
        Expression::MessageSend(
            box MessageSend::Keyword(
                Expression::MessageSend(box MessageSend::Binary(
                    Expression::MessageSend(box MessageSend::Unary(_, _)),              // (((13 x)
                    _,                                                                  // +
                    _,                                                                  // 42)
                )),
                box [
                    (
                        _,                                                              // x:
                        Expression::MessageSend(box MessageSend::Binary(
                            Expression::MessageSend(box MessageSend::Binary(
                                _,                                                      // ((2
                                _,                                                      // +
                                Expression::MessageSend(box MessageSend::Unary(_, _,)), // (2 x))
                            )),
                            _,                                                          // +
                            _,                                                          // 2)
                        ))
                    ),
                    (
                        _,                                                              // x:
                        _,                                                              // 5)
                    )
                ]
            )
        )
    }

    assert_parses! {
        minimal_concrete_method,
        "x => 12",
        parse_concrete_method,
        ConcreteMethod(_, _, _, _)
    }

    assert_parses! {
        maximal_concrete_method,
        "<List<X> a in, Object b out> method: a x with: a y -> Dictionary<X, Y> => 12 + 42",
        parse_concrete_method,
        ConcreteMethod(_, _, _, _)
    }
}
