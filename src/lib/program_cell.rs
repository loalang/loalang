use crate::syntax::*;
use crate::*;
use std::option::NoneError;

pub struct ProgramCell {
    modules: HashMap<URI, ModuleCell>,
    syntax_errors: HashMap<URI, Vec<Diagnostic>>,
    references: HashMap<URI, References>,
}

impl ProgramCell {
    pub fn new() -> ProgramCell {
        ProgramCell {
            modules: HashMap::new(),
            syntax_errors: HashMap::new(),
            references: HashMap::new(),
        }
    }

    pub fn diagnostics(&self) -> Vec<&Diagnostic> {
        self.syntax_errors
            .iter()
            .flat_map(|(_, d)| d.iter())
            .collect()
    }

    pub fn get_source(&self, uri: &URI) -> Option<&Arc<Source>> {
        Some(&self.modules.get(uri)?.source)
    }

    pub fn uris(&self) -> Vec<URI> {
        self.modules.keys().cloned().collect()
    }

    pub fn set(&mut self, source: Arc<Source>) {
        let uri = source.uri.clone();
        let (cell, diagnostics) = ModuleCell::new(source);
        self.modules.insert(uri.clone(), cell);
        self.syntax_errors.insert(uri, diagnostics);
    }

    pub fn replace(&mut self, uri: &URI, new_text: String) -> Result<(), NoneError> {
        match self.modules.get_mut(&uri) {
            None => Err(NoneError),
            Some(cell) => {
                self.syntax_errors
                    .insert(uri.clone(), cell.replace(new_text));
                Ok(())
            }
        }
    }

    pub fn change(&mut self, span: Span, new_text: String) -> Result<(), NoneError> {
        match self.modules.get_mut(&span.start.uri) {
            None => Err(NoneError),
            Some(cell) => {
                self.syntax_errors
                    .insert(span.start.uri.clone(), cell.change(span, new_text));
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
        let references = self.get_references_for_module(&location.uri)?.clone();
        let selection = self.pierce(location);
        let reference = selection.first::<Symbol>()?;
        let declaration_id = references.declaration_of(reference.id)?;
        let declaration = self.find_node(declaration_id)?;
        Some(self.pierce(declaration.span()?.start.clone()))
    }

    pub fn references(&mut self, location: Location) -> Vec<Selection> {
        self.get_references(location).unwrap_or(vec![])
    }

    pub fn find_node(&self, id: Id) -> Option<&dyn Node> {
        for (_, module_cell) in self.modules.iter() {
            match module_cell.find_node(id) {
                None => (),
                Some(n) => return Some(n),
            }
        }
        None
    }

    #[inline]
    fn get_references_for_module(&mut self, uri: &URI) -> Option<&References> {
        if !self.references.contains_key(uri) {
            self.references.insert(
                uri.clone(),
                reference_resolver::get_references(&self.modules.get(uri)?.module),
            );
        }
        self.references.get(uri)
    }

    fn get_references(&mut self, location: Location) -> Option<Vec<Selection>> {
        let references = self.get_references_for_module(&location.uri)?.clone();
        let selection = self.pierce(location.clone());

        if selection.selects_message_pattern_selector() {

        }

        let declaration = selection.first::<Symbol>()?;

        let mut selections = vec![];
        for reference_id in references.references_of(declaration.id) {
            if let Some(reference) = self.find_node(reference_id) {
                if let Some(span) = reference.span() {
                    selections.push(self.pierce(span.start.clone()));
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
        cell.change(Span::at_range(&source, 55..56), "Hello".into())
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
