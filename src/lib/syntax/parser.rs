use crate::syntax::*;
use crate::*;
use TokenKind::*;

macro_rules! consume {
    ($self: expr, $kind: pat) => {{
        let t = $self.tokens[$self.offset].clone();
        if !matches!(t.kind, $kind) {
            return Failure(vec![Diagnostic::UnexpectedToken(
                t,
                stringify!($kind).into(),
            )]);
        }
        $self.offset += 1;
        t
    }};
}

macro_rules! sees {
    ($self: expr, $kind: pat, $ahead: expr) => {{
        let t = &$self.tokens[$self.offset + $ahead];
        matches!(t.kind, $kind)
    }};

    ($self: expr, $kind: pat) => {{
        sees!($self, $kind, 0)
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
        if sees!(self, SimpleSymbol(_)) && !sees!(self, Colon, 1) {
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
}
