use crate::format::*;
use crate::syntax::*;

impl Format for Token {
    fn write(&self, ctx: &mut FormattingContext) {
        ctx.putstr(self.lexeme())
    }
}

impl Format for Integer {
    fn write(&self, ctx: &mut FormattingContext) {
        let Integer(t) = self;
        t.write(ctx);
    }
}

impl Format for Identifier {
    fn write(&self, ctx: &mut FormattingContext) {
        if let Identifier(Token {
            kind: TokenKind::SimpleSymbol(s),
            ..
        }) = self
        {
            ctx.putstr(s);
        } else {
            ctx.putchar('?');
        }
    }
}

impl<T: Format> Format for Keyworded<T> {
    fn write(&self, ctx: &mut FormattingContext) {
        for (i, (keyword, value)) in self.iter().enumerate() {
            if i > 0 {
                if ctx.one_line(self) {
                    ctx.space();
                } else {
                    ctx.break_line();
                }
            }
            keyword.write(ctx);
            ctx.space();
            value.write(ctx);
        }
    }
}

impl Format for MessageSend {
    fn write(&self, ctx: &mut FormattingContext) {
        if ctx.one_line(self) {
            match self {
                MessageSend::Unary(r, id) => {
                    r.write(ctx);
                    ctx.space();
                    id.write(ctx);
                }

                MessageSend::Binary(r, op, a) => {
                    r.write(ctx);
                    ctx.space();
                    op.write(ctx);
                    ctx.space();
                    a.write(ctx);
                }

                MessageSend::Keyword(r, kws) => {
                    r.write(ctx);
                    ctx.space();
                    kws.write(ctx);
                }
            }
        } else {
            match self {
                MessageSend::Unary(r, id) => {
                    r.write(ctx);
                    ctx.indent(move |ctx| {
                        ctx.break_line();
                        id.write(ctx);
                    });
                }

                MessageSend::Binary(r, op, a) => {
                    r.write(ctx);
                    ctx.indent(move |ctx| {
                        ctx.break_line();
                        op.write(ctx);
                        ctx.space();
                        a.write(ctx);
                    });
                }

                MessageSend::Keyword(r, kws) => {
                    r.write(ctx);
                    ctx.indent(move |ctx| {
                        ctx.break_line();
                        kws.write(ctx);
                    });
                }
            }
        }
    }
}

impl Format for Keyword {
    fn write(&self, ctx: &mut FormattingContext) {
        let Keyword(id, _) = self;
        id.write(ctx);
        ctx.putchar(':');
    }
}

impl Format for Expression {
    fn write(&self, ctx: &mut FormattingContext) {
        match self {
            Expression::Integer(i) => i.write(ctx),
            Expression::MessageSend(i) => i.write(ctx),
        }
    }
}
