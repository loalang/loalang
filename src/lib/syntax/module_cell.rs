use crate::syntax::*;
use crate::*;

pub struct ModuleCell {
    pub source: Arc<Source>,
    diagnostics: Vec<Diagnostic>,
    pub module: Module,
}

impl ModuleCell {
    pub fn new(source: Arc<Source>) -> ModuleCell {
        let (diagnostics, module) = Self::parse(&source);

        ModuleCell {
            source,
            diagnostics,
            module,
        }
    }

    fn parse(source: &Arc<Source>) -> (Vec<Diagnostic>, Module) {
        let mut parser = Parser::new(source.clone());
        let module = parser.parse_module();

        (parser.diagnostics, module)
    }

    fn update(&mut self) {
        let (diagnostics, module) = Self::parse(&self.source);
        self.diagnostics = diagnostics;
        self.module = module;
    }

    pub fn change(&mut self, span: Span, new_text: &str) {
        let mut code = self.source.code.clone();
        code.replace_range(span.start.offset..span.end.offset, new_text);
        self.source = Source::new(self.source.uri.clone(), code);
        self.update();
    }
}
