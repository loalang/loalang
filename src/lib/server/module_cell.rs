use crate::syntax::*;
use crate::*;

pub struct ModuleCell {
    pub tree: Arc<Tree>,
    pub source: Arc<Source>,
    pub diagnostics: Vec<Diagnostic>,
}

impl ModuleCell {
    pub fn new(source: Arc<Source>) -> ModuleCell {
        let (tree, diagnostics) = Parser::new(source.clone()).parse();
        ModuleCell {
            tree,
            source,
            diagnostics,
        }
    }

    pub fn edit(&mut self, mut edits: Vec<(Span, String)>) {
        edits.sort_by(|(a, _), (b, _)| a.start.offset.cmp(&b.start.offset));

        let mut current_code = self.source.code.chars().collect::<Vec<_>>();
        let original_len = current_code.len();
        let mut new_code = vec![];

        for (span, code) in edits {
            let number_of_chars_to_append = span.start.offset - (original_len - current_code.len());
            let number_of_chars_to_skip = span.end.offset - span.start.offset;

            for _ in 0..number_of_chars_to_append {
                new_code.push(current_code.remove(0));
            }

            for _ in 0..number_of_chars_to_skip {
                current_code.remove(0);
            }

            new_code.extend(code.chars());
        }

        new_code.extend(current_code);

        let new_code = new_code.into_iter().collect();

        let new_source = Source::new(self.source.uri.clone(), new_code);
        let (tree, diagnostics) = Parser::new(new_source.clone()).parse();

        self.source = new_source;
        self.tree = tree;
        self.diagnostics = diagnostics;
    }
}
