use crate::syntax::*;
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

    pub fn uris(&self) -> Vec<URI> {
        self.modules.keys().cloned().collect()
    }

    pub fn set(&mut self, source: Arc<Source>) {
        let uri = source.uri.clone();
        self.modules.insert(uri, ModuleCell::new(source));
    }

    pub fn get(&self, uri: &URI) -> Option<&ModuleCell> {
        self.modules.get(uri)
    }

    pub fn get_mut(&mut self, uri: &URI) -> Option<&mut ModuleCell> {
        self.modules.get_mut(uri)
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

    pub fn pierce(&self, location: Location) -> Selection {
        match self.modules.get(&location.uri) {
            None => Selection::empty(),
            Some(cell) => cell.pierce(location),
        }
    }

    pub fn declaration(&mut self, location: Location) -> Selection {
        self.get_declaration(location).unwrap_or(Selection::empty())
    }

    fn get_declaration(&mut self, location: Location) -> Option<Selection> {
        let module_cell = self.modules.get_mut(&location.uri)?;
        let references = module_cell.references();
        let selection = module_cell.pierce(location);
        let reference = selection.first::<Symbol>()?;
        let declaration_id = references.declaration_of(reference.id)?;
        let declaration = module_cell.find_node(declaration_id)?;
        Some(module_cell.pierce(declaration.span()?.start.clone()))
    }

    pub fn references(&mut self, location: Location) -> Vec<Selection> {
        self.get_references(location).unwrap_or(vec![])
    }

    fn get_references(&mut self, location: Location) -> Option<Vec<Selection>> {
        let module_cell = self.modules.get_mut(&location.uri)?;
        let references = module_cell.references();
        let selection = module_cell.pierce(location.clone());
        let declaration = selection.first::<Symbol>()?;

        let mut selections = vec![];
        for reference_id in references.references_of(declaration.id) {
            if let Some(reference) = module_cell.find_node(reference_id) {
                if let Some(span) = reference.span() {
                    selections.push(module_cell.pierce(span.start.clone()));
                }
            }
        }
        Some(selections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_cell(code: &str) -> (ProgramCell, Arc<Source>) {
        let mut cell = ProgramCell::new();
        let source = Source::test(code);

        cell.set(source.clone());

        (cell, source)
    }

    #[test]
    fn changing_a_module() {
        let (mut cell, source) = test_cell(
            r#"
                namespace X/Y;

                class Z.
            "#,
        );

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
        let (cell, source) = test_cell(
            r#"
                namespace X/Y;

                class Z.
            "#,
        );

        let location = Location::at_offset(&source, 29);

        let nodes = cell.pierce(location);

        // Symbol
        // QualifiedSymbol
        // NamespaceDirective
        // Module
        assert_eq!(nodes.len(), 4);

        assert_matches!(&nodes.first::<Symbol>().unwrap().token.kind, TokenKind::SimpleSymbol(s) if s == "Y")
    }

    #[test]
    fn declaration() {
        let (mut cell, source) = test_cell(
            r#"
                class A {
                  public a -> A.
                }
            "#,
        );

        let location = Location::at_offset(&source, 57);

        let selection = cell.declaration(location);
        let class = selection.first::<Class>().unwrap();

        assert_eq!(class.name(), "A");
    }

    #[test]
    fn references() {
        let (mut cell, source) = test_cell(
            r#"
                class A {
                  public a -> A.
                  public b -> A.
                }
            "#,
        );

        let location = Location::at_offset(&source, 23);

        let selections = cell.references(location);

        assert_eq!(selections.len(), 2);

        let first_selection = selections.first().unwrap();
        let type_expression = first_selection.first::<TypeExpression>().unwrap();

        assert_matches!(type_expression, TypeExpression::Reference(_, Symbol { .. }));
    }

    #[test]
    fn method_references() {
        let (mut cell, source) = test_cell(
            r#"
                class A {
                  public b -> A.
                  public a: A a -> A => a b.
                }
            "#,
        );

        let location = Location::at_offset(&source, 50);
        let selections = cell.references(location);

        assert_eq!(selections.len(), 1);

        let first_selection = selections.first().unwrap();
        let message = first_selection.first::<Message>().unwrap();

        assert_matches!(message, Message::Unary(_, _));
    }
}
