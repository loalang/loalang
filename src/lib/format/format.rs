use crate::*;

pub struct FormattingContext {
    one_line: bool,
    indentation: usize,
    max_width: usize,
    line_len: usize,
    out: String,
}

impl FormattingContext {
    pub fn one_line(&self, f: &dyn Format) -> bool {
        self.one_line || (self.line_len + f.format_one_line().len()) < self.max_width
    }

    pub fn indent<F: FnOnce(&mut FormattingContext)>(&mut self, f: F) {
        self.indent_start();
        f(self);
        self.indent_end();
    }

    pub fn indent_start(&mut self) {
        self.indentation += 1;
    }

    pub fn indent_end(&mut self) {
        self.indentation -= 1;
    }

    pub fn break_line(&mut self) {
        self.putchar('\n');
        self.line_len = 0;
        for _ in 0..self.indentation {
            self.putstr("  ");
        }
    }

    pub fn putchar(&mut self, c: char) {
        self.line_len += 1;
        self.out.push(c);
    }

    pub fn putstr<S: AsRef<str>>(&mut self, s: S) {
        let s = s.as_ref();
        self.line_len += s.len();
        self.out.push_str(s);
    }

    pub fn space(&mut self) {
        self.putchar(' ');
    }

    pub fn type_var_list<V: Format, F: Fn(&mut FormattingContext, &V)>(
        &mut self,
        vs: &Vec<V>,
        f: F,
    ) {
        if vs.len() > 0 {
            self.putchar('<');
            self.list(vs, f);
            self.putchar('>');
        }
    }

    pub fn array_list<V: Format, F: Fn(&mut FormattingContext, &V)>(&mut self, vs: &Vec<V>, f: F) {
        self.putchar('[');
        self.list(vs, f);
        self.putchar(']');
    }

    fn list<V: Format, F: Fn(&mut FormattingContext, &V)>(&mut self, vs: &Vec<V>, f: F) {
        for (i, v) in vs.iter().enumerate() {
            if i > 0 {
                self.putstr(", ");
            }

            f(self, v);
        }
    }
}

pub trait Format {
    fn write(&self, ctx: &mut FormattingContext);

    fn format(&self) -> String {
        let mut ctx = FormattingContext {
            one_line: false,
            indentation: 0,
            line_len: 0,
            max_width: 90,
            out: String::new(),
        };
        self.write(&mut ctx);
        ctx.out
    }

    fn format_one_line(&self) -> String {
        let mut ctx = FormattingContext {
            one_line: true,
            indentation: 0,
            line_len: 0,
            max_width: 0,
            out: String::new(),
        };
        self.write(&mut ctx);
        ctx.out
    }
}

impl<T: Format> Format for Arc<T> {
    fn write(&self, ctx: &mut FormattingContext) {
        (self as &T).write(ctx);
    }
}

impl fmt::Display for dyn Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}
