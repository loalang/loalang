use crate::*;

pub struct FormattingContext {
    one_line: bool,
    indentation: usize,
    max_width: usize,
}

impl FormattingContext {
    pub fn one_line(&self, f: &Format) -> bool {
        self.one_line || f.format_one_line().len() < self.max_width
    }

    pub fn indent<F: FnOnce(&mut FormattingContext)>(&mut self, f: F) {
        self.indentation += 1;
        f(self);
        self.indentation -= 1;
    }

    pub fn break_line(&self, out: &mut String) {
        out.push('\n');
        for _ in 0..self.indentation {
            out.push_str("  ");
        }
    }
}

pub trait Format {
    fn write(&self, out: &mut String, ctx: &mut FormattingContext);

    fn format(&self, ctx: &mut FormattingContext) -> String {
        let mut out = String::new();
        self.write(&mut out, ctx);
        out
    }

    fn format_one_line(&self) -> String {
        let mut out = String::new();
        self.write(
            &mut out,
            &mut FormattingContext {
                one_line: true,
                indentation: 0,
                max_width: 0,
            },
        );
        out
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            self.format(&mut FormattingContext {
                one_line: false,
                indentation: 0,
                max_width: 90,
            })
        )
    }
}
