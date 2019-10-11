use crate::syntax::ModuleCell;
use crate::*;
use std::option::NoneError;

pub struct ProgramCell {
    modules: HashMap<URI, ModuleCell>,
}

impl ProgramCell {
    pub fn new() -> ProgramCell {
        ProgramCell {
            modules: HashMap::new(),
        }
    }

    pub fn set(&mut self, source: Arc<Source>) {
        let uri = source.uri.clone();
        self.modules.insert(uri, ModuleCell::new(source));
    }

    pub fn change(&mut self, span: Span, new_text: &str) -> Result<(), NoneError> {
        match self.modules.get_mut(&span.start.uri) {
            None => Err(NoneError),
            Some(cell) => {
                cell.change(span, new_text);
                Ok(())
            }
        }
    }

    pub fn pierce(&self, location: Location) -> Vec<&dyn syntax::Node> {
        match self.modules.get(&location.uri) {
            None => vec![],
            Some(cell) => {
                let mut nodes = vec![];
                for node in syntax::traverse(&cell.module) {
                    if node.is_token() {
                        continue;
                    }

                    if node.contains_location(&location) {
                        nodes.push(node);
                    }
                }
                nodes
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changing_a_module() {
        let mut cell = ProgramCell::new();
        let source = Source::test(
            r#"
                namespace X/Y;

                class Z.
            "#,
        );

        cell.set(source.clone());

        // Rename class
        cell.change(Span::at_range(&source, 55..56), "Hello")
            .unwrap();

        assert_eq!(
            cell.modules[&source.uri].source.code,
            r#"
                namespace X/Y;

                class Hello.
            "#
        );
    }

    #[test]
    fn pierce() {
        let mut cell = ProgramCell::new();
        let source = Source::test(
            r#"
                namespace X/Y;

                class Z.
            "#,
        );

        cell.set(source.clone());

        let location = Location::at_offset(&source, 29);

        let nodes = cell.pierce(location);

        // Symbol
        // QualifiedSymbol
        // NamespaceDirective
        // Module
        assert_eq!(nodes.len(), 4);
    }
}
