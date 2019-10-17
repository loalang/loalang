use crate::syntax::*;
use crate::*;

pub struct ModuleCell {
    pub source: Arc<Source>,
    pub module: Module,
}

impl ModuleCell {
    pub fn new(source: Arc<Source>) -> (ModuleCell, Vec<Diagnostic>) {
        let (diagnostics, module) = Self::parse(&source);

        (ModuleCell { source, module }, diagnostics)
    }

    fn parse(source: &Arc<Source>) -> (Vec<Diagnostic>, Module) {
        let mut parser = Parser::new(source.clone());
        let module = parser.parse_module();

        (parser.diagnostics, module)
    }

    fn update(&mut self) -> Vec<Diagnostic> {
        let (diagnostics, module) = Self::parse(&self.source);
        self.module = module;
        diagnostics
    }

    pub fn replace(&mut self, code: String) -> Vec<Diagnostic> {
        self.source = Source::new(self.source.uri.clone(), code);
        self.update()
    }

    pub fn change(&mut self, span: Span, new_text: String) -> Vec<Diagnostic> {
        let mut code = self.source.code.clone();
        code.replace_range(span.start.offset..span.end.offset, new_text.as_ref());
        self.source = Source::new(self.source.uri.clone(), code);
        self.update()
    }

    pub fn pierce(&self, location: Location) -> Selection {
        let mut nodes = vec![];
        for node in traverse(&self.module) {
            if node.is_token() {
                continue;
            }

            if node.contains_location(&location) {
                nodes.push(node);
            }
        }
        Selection::new(nodes)
    }

    pub fn find_node(&self, id: Id) -> Option<&dyn Node> {
        for node in traverse(&self.module) {
            if let Some(nid) = node.id() {
                if nid == id {
                    return Some(node);
                }
            }
        }
        None
    }
}
