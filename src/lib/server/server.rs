use crate::semantics::{Analysis, Navigator};
use crate::syntax::DeclarationKind;
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
            edits.push((span, code));
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

    pub fn declaration_is_exported(&self, declaration: &syntax::Node) -> bool {
        self.analysis.declaration_is_exported(declaration)
    }

    pub fn usage(&mut self, location: Location) -> Option<server::Usage> {
        let cell = self.module_cells.get(&location.uri)?;
        let node = cell.tree.node_at(location)?;
        if !node.is_symbol() {
            return None;
        }
        let navigator = self.analysis.navigator();
        if let Some(qs) = navigator.parent(&node) {
            if let syntax::QualifiedSymbol { ref symbols } = qs.kind {
                if symbols.last() != Some(&node.id) {
                    return None;
                }
            }
        }
        let usage = self.analysis.usage(node)?;
        Some(server::Usage {
            handle: self.create_named_node(&node)?,
            declaration: self.create_named_node(&usage.declaration)?,
            references: usage
                .references
                .iter()
                .filter_map(|n| self.create_named_node(n))
                .collect(),
            imports: usage
                .import_directives
                .iter()
                .flat_map(|import| {
                    let mut named_nodes = vec![];
                    if let syntax::ImportDirective {
                        qualified_symbol,
                        symbol,
                        ..
                    } = import.kind
                    {
                        if let Some(symbol) = navigator.find_node(symbol) {
                            if let Some(mut named_node) = self.create_named_node(&symbol) {
                                named_node.node = import.clone();
                                named_nodes.push(named_node);
                            }
                        }

                        if let Some(qs) = navigator.find_node(qualified_symbol) {
                            if let syntax::QualifiedSymbol { symbols } = qs.kind {
                                if let Some(last_symbol) =
                                    symbols.last().cloned().and_then(|i| navigator.find_node(i))
                                {
                                    if let Some(mut named_node) =
                                        self.create_named_node(&last_symbol)
                                    {
                                        named_node.node = import.clone();
                                        named_nodes.push(named_node);
                                    }
                                }
                            }
                        }
                    }
                    named_nodes
                })
                .collect(),
        })
    }

    fn create_named_node(&self, node: &syntax::Node) -> Option<server::NamedNode> {
        let navigator = semantics::ModuleNavigator::new(
            self.module_cells.get(&node.span.start.uri)?.tree.clone(),
        );
        let (name, symbol) = navigator.symbol_of(node)?;
        Some(server::NamedNode {
            name_span: symbol.span.clone(),
            name,
            node: node.clone(),
        })
    }

    pub fn completion(&mut self, location: Location) -> Option<server::Completion> {
        let tree = &self.module_cells.get(&location.uri)?.tree;
        let (before, _at, _after) = tree.nodes_around(location);

        let mut before = before?.clone();

        if before.is_symbol() {
            before = tree.get(before.parent_id?)?;
        }

        if before.is_message() {
            before = tree.get(before.parent_id?)?;
        }

        match before.kind {
            _ if before.is_expression() => {
                let type_ = self.analysis.types.get_type_of_expression(&before);

                Some(server::Completion::Behaviours(
                    self.analysis.types.get_behaviours(&type_),
                ))
            }

            syntax::KeywordPair { .. } => {
                // A keyword pair can occur in different locations in the tree.
                // It can be part of a message, or a message pattern.
                // Looking at the parent can give us some clue.
                let parent = tree.get(before.parent_id?)?;

                match parent.kind {
                    syntax::KeywordMessage { .. } => {
                        self.completion_on_declarations_in_scope(&before, DeclarationKind::Value)
                    }
                    syntax::KeywordMessagePattern { .. } => {
                        self.completion_on_declarations_in_scope(&before, DeclarationKind::Type)
                    }
                    kind => {
                        warn!(
                            "Cannot get completion from a keyword pair within {:?}",
                            kind
                        );
                        None
                    }
                }
            }

            syntax::MethodBody { .. } => {
                self.completion_on_declarations_in_scope(&before, DeclarationKind::Value)
            }

            kind => {
                warn!("Cannot get completion on {:?}", kind);
                None
            }
        }
    }

    fn completion_on_declarations_in_scope(
        &self,
        from: &syntax::Node,
        kind: DeclarationKind,
    ) -> Option<server::Completion> {
        let declarations = self.analysis.declarations_in_scope(from.clone(), kind);

        Some(server::Completion::VariablesInScope(
            declarations
                .into_iter()
                .filter_map(|(name, dec)| {
                    Some(server::Variable {
                        name,
                        type_: server::Type::Unknown,
                        kind: match dec.kind {
                            syntax::Class { .. } => server::VariableKind::Class,
                            syntax::ParameterPattern { .. } => server::VariableKind::Parameter,
                            _ => server::VariableKind::Unknown,
                        },
                    })
                })
                .collect(),
        ))
    }
}
