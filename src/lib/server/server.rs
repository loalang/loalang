use crate::semantics::Analysis;
use crate::*;

pub struct Server {
    analysis: semantics::Analysis,
    module_cells: HashMap<URI, server::ModuleCell>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            analysis: semantics::Analysis::new(Arc::new(HashMap::new())),
            module_cells: HashMap::new(),
        }
    }

    /// Sweep the entire program for all diagnostics,
    /// syntax errors and semantics.
    pub fn diagnostics(&mut self) -> HashMap<URI, Vec<Diagnostic>> {
        let mut all: HashMap<URI, Vec<Diagnostic>> = self
            .module_cells
            .iter()
            .map(|(uri, _)| (uri.clone(), vec![]))
            .collect();
        for (uri, cell) in self.module_cells.iter() {
            all.get_mut(uri)
                .unwrap()
                .extend(cell.diagnostics.iter().cloned());
        }
        for diagnostic in self.analysis.check() {
            if let Some(d) = all.get_mut(&diagnostic.span().start.uri) {
                d.push(diagnostic);
            }
        }
        all
    }

    // SOURCE CODE MANIPULATION

    fn reset_analysis(&mut self) {
        let mut modules = HashMap::new();
        for (uri, cell) in self.module_cells.iter() {
            modules.insert(uri.clone(), cell.tree.clone());
        }
        self.analysis = Analysis::new(Arc::new(modules));
    }

    pub fn set(&mut self, uri: URI, code: String) {
        self.module_cells
            .insert(uri.clone(), server::ModuleCell::new(Source::new(uri, code)));
        self.reset_analysis();
    }

    pub fn remove(&mut self, uri: URI) {
        self.module_cells.remove(&uri);
        self.reset_analysis();
    }

    pub fn edit(&mut self, edits: Vec<(Span, String)>) {
        let mut loa_edits = HashMap::new();
        for (span, code) in edits {
            if !loa_edits.contains_key(&span.start.uri) {
                loa_edits.insert(span.start.uri.clone(), vec![]);
            }
            let edits = loa_edits.get_mut(&span.start.uri).unwrap();
            if let Some(cell) = self.module_cells.get(&span.start.uri) {
                edits.push((span, code));
            }
        }
        for (uri, edits) in loa_edits {
            self.module_cells.get_mut(&uri).map(|cell| cell.edit(edits));
        }
        self.reset_analysis();
    }

    // RESOLVE LOCATION
    pub fn source(&self, uri: &URI) -> Option<Arc<Source>> {
        let cell = self.module_cells.get(uri)?;
        Some(cell.source.clone())
    }

    pub fn tree(&self, uri: &URI) -> Option<Arc<syntax::Tree>> {
        let cell = self.module_cells.get(uri)?;
        Some(cell.tree.clone())
    }

    pub fn span(&self, uri: &URI, (from, to): ((usize, usize), (usize, usize))) -> Option<Span> {
        Some(Span::new(
            self.location(uri, from)?,
            self.location(uri, to)?,
        ))
    }

    pub fn location(&self, uri: &URI, (line, character): (usize, usize)) -> Option<Location> {
        Location::at_position(&self.module_cells.get(uri)?.source, line, character)
    }

    // SEMANTIC QUERIES

    pub fn usage(&mut self, location: Location) -> Option<server::Usage> {
        let symbol = self.module_cells.get(&location.uri)?;
        let node = symbol.tree.node_at(location)?;
        let usage = self.analysis.usage(node)?;
        Some(server::Usage {
            declaration: self.create_named_node(&usage.declaration)?,
            references: usage
                .references
                .iter()
                .filter_map(|n| self.create_named_node(n))
                .collect(),
        })
    }

    fn create_named_node(&self, node: &syntax::Node) -> Option<server::NamedNode> {
        let symbol = self.find_node(&node.span.start.uri, node.symbol_id()?)?;
        if let syntax::Symbol(ref t) = symbol.kind {
            Some(server::NamedNode {
                name_span: t.span.clone(),
                name: t.lexeme(),
                node: node.clone(),
            })
        } else {
            None
        }
    }

    #[inline]
    fn find_node(&self, uri: &URI, id: Id) -> Option<syntax::Node> {
        self.module_cells.get(uri)?.tree.get(id)
    }

    pub fn completion(&mut self, location: Location) -> Option<server::Completion> {
        None
    }
}
