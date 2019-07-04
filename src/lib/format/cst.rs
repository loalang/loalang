use crate::format::*;
use crate::syntax::*;

impl Format for Token {
    fn write(&self, out: &mut String, _ctx: &mut FormattingContext) {
        use TokenKind::*;

        match &self.kind {
            EOF => (),
            Unknown(c) => out.push(*c),

            Plus => out.push('+'),
            Colon => out.push(':'),

            Whitespace(s) | LineComment(s) | SimpleInteger(s) | SimpleSymbol(s) => {
                out.push_str(s.as_str())
            }
        }
    }
}

impl Format for Integer {
    fn write(&self, out: &mut String, ctx: &mut FormattingContext) {
        let Integer(t) = self;
        t.write(out, ctx);
    }
}

impl Format for Identifier {
    fn write(&self, out: &mut String, _ctx: &mut FormattingContext) {
        if let Identifier(Token {
            kind: TokenKind::SimpleSymbol(s),
            ..
        }) = self
        {
            out.push_str(s.as_ref());
        } else {
            out.push('?');
        }
    }
}

impl<T: Format> Format for Keyworded<T> {
    fn write(&self, out: &mut String, ctx: &mut FormattingContext) {
        for (i, (keyword, value)) in self.iter().enumerate() {
            if i > 0 {
                if ctx.one_line(self) {
                    out.push(' ');
                } else {
                    ctx.break_line(out);
                }
            }
            keyword.write(out, ctx);
            out.push(' ');
            value.write(out, ctx);
        }
    }
}

impl Format for MessageSend {
    fn write(&self, out: &mut String, ctx: &mut FormattingContext) {
        if ctx.one_line(self) {
            match self {
                MessageSend::Unary(r, id) => {
                    r.write(out, ctx);
                    out.push(' ');
                    id.write(out, ctx);
                }

                MessageSend::Binary(r, op, a) => {
                    r.write(out, ctx);
                    out.push(' ');
                    op.write(out, ctx);
                    out.push(' ');
                    a.write(out, ctx);
                }

                MessageSend::Keyword(r, kws) => {
                    r.write(out, ctx);
                    out.push(' ');
                    kws.write(out, ctx);
                }
            }
        } else {
            match self {
                MessageSend::Unary(r, id) => {
                    r.write(out, ctx);
                    ctx.indent(move |ctx| {
                        ctx.break_line(out);
                        id.write(out, ctx);
                    });
                }

                MessageSend::Binary(r, op, a) => {
                    r.write(out, ctx);
                    ctx.indent(move |ctx| {
                        ctx.break_line(out);
                        op.write(out, ctx);
                        out.push(' ');
                        a.write(out, ctx);
                    });
                }

                MessageSend::Keyword(r, kws) => {
                    r.write(out, ctx);
                    ctx.indent(move |ctx| {
                        ctx.break_line(out);
                        kws.write(out, ctx);
                    });
                }
            }
        }
    }
}

impl Format for Keyword {
    fn write(&self, out: &mut String, ctx: &mut FormattingContext) {
        let Keyword(id, _) = self;
        id.write(out, ctx);
        out.push(':');
    }
}

impl Format for Expression {
    fn write(&self, out: &mut String, ctx: &mut FormattingContext) {
        match self {
            Expression::Integer(i) => i.write(out, ctx),
            Expression::MessageSend(i) => i.write(out, ctx),
        }
    }
}
