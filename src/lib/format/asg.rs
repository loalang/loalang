use crate::format::*;
use crate::semantics::*;

impl Format for Expression {
    fn write(&self, ctx: &mut FormattingContext) {
        match self {
            Expression::Integer(i) => ctx.putstr(if i < &100000.into() {
                format!("{}", i.to_str_radix(10))
            } else {
                format!("32#{}", i.to_str_radix(32))
            }),
            Expression::MessageSend(r, m) => {
                ctx.putchar('(');
                r.write(ctx);
                if ctx.one_line(self) {
                    ctx.space();
                    m.write(ctx);
                } else {
                    ctx.indent(|ctx| {
                        ctx.break_line();
                        m.write(ctx);
                    });
                    ctx.break_line();
                }
                ctx.putchar(')');
            }
        }
    }
}

impl Format for Message {
    fn write(&self, ctx: &mut FormattingContext) {
        match self.arguments.len() {
            0 => {
                self.selector.write(ctx);
            }

            1 => {
                self.selector.write(ctx);
                ctx.space();
                self.arguments[0].write(ctx);
            }

            _ => self
                .arguments
                .iter()
                .zip(self.selector.to_string().split(':'))
                .enumerate()
                .for_each(|(i, (arg, kw))| {
                    if ctx.one_line(self) {
                        if i > 0 {
                            ctx.space();
                        }
                        ctx.putstr(kw);
                        ctx.putstr(": ");
                        arg.write(ctx);
                    } else {
                        if i > 0 {
                            ctx.break_line();
                        }
                        ctx.putstr(kw);
                        ctx.putstr(": ");
                        arg.write(ctx);
                    }
                }),
        }
    }
}

impl Format for Class {
    fn write(&self, ctx: &mut FormattingContext) {
        ctx.putstr("class ");

        self.name.write(ctx);
        ctx.type_var_list(&self.type_parameters, |ctx, tp| {
            tp.write(ctx);
        });

        ctx.putstr(" {");
        if ctx.one_line(self) {
            ctx.space();
            for method in self.methods.iter() {
                method.write(ctx);
                ctx.space();
            }
        } else if self.methods.len() > 0 {
            ctx.indent(|ctx| {
                ctx.break_line();
                for (i, method) in self.methods.iter().enumerate() {
                    if i > 0 {
                        ctx.break_line();
                        ctx.break_line();
                    }
                    method.visibility.write(ctx);
                    ctx.space();
                    method.write(ctx);
                    ctx.putchar('.');
                }
            });
            ctx.break_line();
        }
        ctx.putchar('}');
    }
}

impl Format for Visibility {
    fn write(&self, ctx: &mut FormattingContext) {
        match self {
            Visibility::Public => ctx.putstr("public"),
            Visibility::Private => ctx.putstr("private"),
        }
    }
}

impl Format for Symbol {
    fn write(&self, ctx: &mut FormattingContext) {
        let Symbol(s) = self;
        ctx.putstr(s);
    }
}

impl Format for TypeParameter {
    fn write(&self, ctx: &mut FormattingContext) {
        self.constraint.write(ctx);
        ctx.space();
        self.name.write(ctx);
        ctx.space();
        self.variance.write(ctx);
    }
}

impl Format for Variance {
    fn write(&self, ctx: &mut FormattingContext) {
        match self {
            Variance::Invariant => ctx.putstr("inout"),
            Variance::Covariant => ctx.putstr("out"),
            Variance::Contravariant => ctx.putstr("in"),
        }
    }
}

impl Format for Method {
    fn write(&self, ctx: &mut FormattingContext) {
        match &self.implementation {
            None => {
                self.signature.write(ctx);
            }
            Some(MethodImplementation::Body(patterns, body)) => {
                let one_line = ctx.one_line(&self.signature);
                ctx.type_var_list(&self.signature.type_parameters, |ctx, p| {
                    p.write(ctx);
                });
                if one_line {
                    if self.signature.type_parameters.len() > 0 {
                        ctx.space();
                    }
                } else {
                    ctx.indent_start();
                    ctx.break_line();
                }
                match patterns.len() {
                    0 => {
                        self.signature.selector.write(ctx);
                    }
                    1 => {
                        self.signature.selector.write(ctx);
                        ctx.space();
                        patterns[0].write(ctx);
                    }
                    _ => patterns
                        .iter()
                        .zip(self.signature.selector.to_string().split(':'))
                        .enumerate()
                        .for_each(|(i, (pat, kw))| {
                            if one_line {
                                if i > 0 {
                                    ctx.space();
                                }
                                ctx.putstr(kw);
                                ctx.putstr(": ");
                                pat.write(ctx);
                            } else {
                                if i > 0 {
                                    ctx.break_line();
                                }
                                ctx.putstr(kw);
                                ctx.putstr(": ");
                                pat.write(ctx);
                            }
                        }),
                }
                ctx.putstr(" -> ");
                self.signature.return_type.write(ctx);
                if ctx.one_line(body) {
                    ctx.putstr(" => ");
                    body.write(ctx);
                } else {
                    ctx.putstr(" =>");
                    ctx.indent(|ctx| {
                        ctx.break_line();
                        body.write(ctx);
                    })
                }
                if !one_line {
                    ctx.indent_end();
                }
            }
            Some(MethodImplementation::VariableGetter(_)) => {}
            Some(MethodImplementation::VariableSetter(_)) => {}
        }
    }
}

impl Format for Signature {
    fn write(&self, ctx: &mut FormattingContext) {
        let one_line = ctx.one_line(self);
        ctx.type_var_list(&self.type_parameters, |ctx, p| {
            p.write(ctx);
        });
        if one_line {
            if self.type_parameters.len() > 0 {
                ctx.space();
            }
        } else {
            ctx.indent_start();
            ctx.break_line();
        }
        match self.parameters.len() {
            0 => {
                self.selector.write(ctx);
            }
            1 => {
                self.selector.write(ctx);
                ctx.space();
                self.parameters[0].write(ctx);
            }
            _ => self
                .parameters
                .iter()
                .zip(self.selector.to_string().split(':'))
                .enumerate()
                .for_each(|(i, (t, kw))| {
                    if one_line {
                        if i > 0 {
                            ctx.space();
                        }
                        ctx.putstr(kw);
                        ctx.putstr(": ");
                        t.write(ctx);
                    } else {
                        if i > 0 {
                            ctx.break_line();
                        }
                        ctx.putstr(kw);
                        ctx.putstr(": ");
                        t.write(ctx);
                    }
                }),
        }
        ctx.putstr(" -> ");
        self.return_type.write(ctx);
        if !one_line {
            ctx.indent_end();
        }
    }
}

impl Format for Pattern {
    fn write(&self, ctx: &mut FormattingContext) {
        match self {
            Pattern::Binding(t, id) => {
                t.write(ctx);
                ctx.space();
                id.write(ctx);
            }
        }
    }
}

impl Format for Type {
    fn write(&self, ctx: &mut FormattingContext) {
        self.constructor.name().write(ctx);
        ctx.type_var_list(&self.arguments, |ctx, a| {
            a.write(ctx);
        });
    }
}
