use crate::semantics::*;
use crate::syntax::DeclarationKind;
use crate::*;

#[derive(Clone)]
pub struct Server {
    pub analysis: semantics::Analysis,
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

    pub fn type_at(&self, location: Location) -> semantics::Type {
        let cell = self.module_cells.get(&location.uri)?;
        let node = cell.tree.node_at(location)?;
        if let Some(expression) = self.analysis.navigator.closest_expression_upwards(node) {
            return self.analysis.types.get_type_of_expression(&expression);
        }
        if let Some(type_expression) = self
            .analysis
            .navigator
            .closest_type_expression_upwards(node)
        {
            return self
                .analysis
                .types
                .get_type_of_type_expression(&type_expression);
        }
        semantics::Type::Unknown
    }

    pub fn behaviour_at(&self, location: Location) -> Option<semantics::Behaviour> {
        let cell = self.module_cells.get(&location.uri)?;
        let node = cell.tree.node_at(location)?;
        if let Some(message_send) = self.analysis.navigator.closest_message_send_upwards(node) {
            return self
                .analysis
                .types
                .get_behaviour_from_message_send(&message_send);
        }
        if let Some(message_pattern) = self
            .analysis
            .navigator
            .closest_message_pattern_upwards(node)
        {
            let signature = self.analysis.navigator.parent(&message_pattern)?;
            let method = self.analysis.navigator.parent(&signature)?;
            let class_body = self.analysis.navigator.parent(&method)?;
            let class = self.analysis.navigator.parent(&class_body)?;

            let receiver_type = self.analysis.types.get_type_of_declaration(&class);

            return self
                .analysis
                .types
                .get_behaviour_from_method(receiver_type, method);
        }
        None
    }

    pub fn usage(&mut self, location: Location) -> Option<server::Usage> {
        let cell = self.module_cells.get(&location.uri)?;
        let node = cell.tree.node_at(location.clone())?;
        if !node.is_symbol() && !node.is_operator() {
            return None;
        }
        if let Some(qs) = self.analysis.navigator.parent(&node) {
            if let syntax::QualifiedSymbol { ref symbols } = qs.kind {
                if symbols.last() != Some(&node.id) {
                    return None;
                }
            }
        }
        let usage =
            self.analysis
                .navigator
                .find_usage(node, DeclarationKind::Any, &self.analysis.types)?;

        let mut handle = None;

        for reference in usage.references.iter() {
            if reference.span.contains_location(&location) {
                handle = Some(reference);
            }
        }

        if handle.is_none() && usage.declaration.span.contains_location(&location) {
            handle = Some(&usage.declaration);
        }

        Some(server::Usage {
            handle: self.create_named_node(handle.unwrap_or(node))?,
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
                        if let Some(symbol) = self.analysis.navigator.find_node(symbol) {
                            if let Some(mut named_node) = self.create_named_node(&symbol) {
                                named_node.node = import.clone();
                                named_nodes.push(named_node);
                            }
                        }

                        if let Some(qs) = self.analysis.navigator.find_node(qualified_symbol) {
                            if let syntax::QualifiedSymbol { symbols } = qs.kind {
                                if let Some(last_symbol) = symbols
                                    .last()
                                    .cloned()
                                    .and_then(|i| self.analysis.navigator.find_node(i))
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
        let navigator = &self.analysis.navigator;

        if node.is_message() {
            return Some(server::NamedNode {
                name_span: node.span.clone(),
                name: navigator.message_selector(&node)?,
                node: node.clone(),
            });
        }

        if let syntax::Method { signature, .. } = node.kind {
            let signature = navigator.find_child(node, signature)?;
            if let syntax::Signature {
                message_pattern, ..
            } = signature.kind
            {
                let message_pattern = navigator.find_child(&signature, message_pattern)?;
                return Some(server::NamedNode {
                    name_span: signature.span.clone(),
                    name: navigator.message_selector(&message_pattern)?,
                    node: node.clone(),
                });
            }
        }

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
                    syntax::KeywordMessage { .. } => self.completion_on_declarations_in_scope(
                        &before,
                        DeclarationKind::Value,
                        &self.analysis.types,
                    ),
                    syntax::KeywordMessagePattern { .. } => self
                        .completion_on_declarations_in_scope(
                            &before,
                            DeclarationKind::Type,
                            &self.analysis.types,
                        ),
                    kind => {
                        warn!(
                            "Cannot get completion from a keyword pair within {:?}",
                            kind
                        );
                        None
                    }
                }
            }

            syntax::MethodBody { .. } => self.completion_on_declarations_in_scope(
                &before,
                DeclarationKind::Value,
                &self.analysis.types,
            ),

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
        types: &semantics::Types,
    ) -> Option<server::Completion> {
        let declarations = self.analysis.declarations_in_scope(from.clone(), kind);

        Some(server::Completion::VariablesInScope(
            declarations
                .into_iter()
                .filter_map(|(name, dec)| {
                    Some(server::Variable {
                        name,
                        type_: types.get_type_of_declaration(&dec),
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
