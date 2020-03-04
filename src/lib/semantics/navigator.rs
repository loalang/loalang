use crate::semantics::*;
use crate::syntax::*;
use crate::*;

#[derive(Clone)]
pub struct Navigator {
    modules: Arc<HashMap<URI, Arc<Tree>>>,

    usage_cache: Cache<(DeclarationKind, Id), Option<Arc<Usage>>>,
    stdlib_class_cache: Cache<String, Option<Node>>,
    stdlib_classes_cache: Cache<(), HashMap<String, Node>>,
}

impl Navigator {
    pub fn new(modules: Arc<HashMap<URI, Arc<Tree>>>) -> Navigator {
        Navigator {
            modules,
            usage_cache: Cache::new(),
            stdlib_class_cache: Cache::new(),
            stdlib_classes_cache: Cache::new(),
        }
    }

    pub fn traverse_all<F: FnMut(&Node) -> bool>(&self, f: &mut F) {
        for module in self.modules.values() {
            if let Some(root) = module.root() {
                self.traverse(root, f);
            }
        }
    }

    pub fn traverse_all_repl_lines<F: FnMut(&Node) -> bool>(&self, f: &mut F) {
        for module in self.modules.values() {
            if let Some(root) = module.root() {
                if root.is_repl_line() {
                    self.traverse(root, f);
                }
            }
        }
    }

    pub fn source(&self, uri: &URI) -> Option<Arc<Source>> {
        self.modules.get(uri).map(|t| t.source.clone())
    }

    pub fn sources(&self) -> Vec<Arc<Source>> {
        self.modules
            .iter()
            .map(|(_, t)| &t.source)
            .cloned()
            .collect()
    }

    pub fn all_top_level_declarations(&self) -> Vec<Node> {
        self.modules
            .iter()
            .filter_map(|(_, tree)| tree.root())
            .flat_map(|root| match root.kind {
                Module { .. } => self
                    .module_declarations_in(root)
                    .into_iter()
                    .map(|(_, n)| n)
                    .collect(),
                REPLLine { .. } => self.declarations_of_repl_line(root),
                _ => vec![],
            })
            .collect()
    }

    pub fn declarations_of_repl_line(&self, repl_line: &Node) -> Vec<Node> {
        match repl_line.kind {
            REPLLine { ref statements, .. } => statements
                .iter()
                .filter_map(|i| self.find_child(repl_line, *i))
                .filter_map(|child| {
                    if child.is_declaration(DeclarationKind::Any) {
                        Some(child)
                    } else {
                        None
                    }
                })
                .collect(),
            _ => vec![],
        }
    }

    pub fn modules(&self) -> Vec<Node> {
        self.modules
            .values()
            .filter_map(|t| t.root())
            .cloned()
            .collect()
    }

    pub fn find_node(&self, id: Id) -> Option<Node> {
        for (_, tree) in self.modules.iter() {
            if let Some(n) = tree.get(id) {
                return Some(n);
            }
        }
        None
    }

    pub fn find_node_in(&self, uri: &URI, id: Id) -> Option<Node> {
        self.modules.get(uri).and_then(|t| t.get(id))
    }

    pub fn parent(&self, child: &Node) -> Option<Node> {
        let uri = &child.span.start.uri;
        child.parent_id.and_then(|i| self.find_node_in(uri, i))
    }

    pub fn children(&self, parent: &Node) -> Vec<Node> {
        let uri = &parent.span.start.uri;
        parent
            .kind
            .children()
            .into_iter()
            .filter_map(|i| self.find_node_in(uri, i))
            .collect()
    }

    pub fn message_selector(&self, message: &Node) -> Option<String> {
        match message.kind {
            UnaryMessage { symbol } | UnaryMessagePattern { symbol } => {
                Some(self.symbol_of(&self.find_child(message, symbol)?)?.0)
            }
            BinaryMessage { operator, .. } | BinaryMessagePattern { operator, .. } => {
                if let Operator(ts) = self.find_child(message, operator)?.kind {
                    Some(
                        ts.into_iter()
                            .map(|ref t| t.lexeme())
                            .collect::<Vec<_>>()
                            .join(""),
                    )
                } else {
                    None
                }
            }
            KeywordMessage { ref keyword_pairs } | KeywordMessagePattern { ref keyword_pairs } => {
                let mut selector = String::new();

                for pair in keyword_pairs.iter() {
                    if let KeywordPair { keyword, .. } = self.find_child(message, *pair)?.kind {
                        selector.push_str(
                            format!(
                                "{}:",
                                self.symbol_of(&self.find_child(message, keyword)?)?.0
                            )
                            .as_ref(),
                        );
                    }
                }

                Some(selector)
            }
            _ => None,
        }
    }

    pub fn message_arguments(&self, message: &Node) -> Vec<Node> {
        match message.kind {
            KeywordMessage {
                ref keyword_pairs, ..
            } => self
                .keyword_pairs(message, keyword_pairs)
                .into_iter()
                .map(|(_, v)| v)
                .collect(),
            BinaryMessage { expression, .. } => {
                self.find_child(message, expression).into_iter().collect()
            }
            UnaryMessage { .. } | _ => vec![],
        }
    }

    pub fn symbol_of(&self, node: &Node) -> Option<(String, Node)> {
        match node.kind {
            Symbol(ref t) => Some((t.lexeme(), node.clone())),
            Operator(ref ts) => Some((
                ts.iter().map(|t| t.lexeme()).collect::<Vec<_>>().join(""),
                node.clone(),
            )),
            SelfTypeExpression { .. } | SelfExpression { .. } => {
                Some(("self".into(), node.clone()))
            }
            Class { symbol, .. }
            | LetBinding { symbol, .. }
            | ReferenceTypeExpression { symbol, .. }
            | ReferenceExpression { symbol, .. }
            | TypeParameter { symbol, .. }
            | ParameterPattern { symbol, .. }
            | Variable { symbol, .. } => self.find_node(symbol).and_then(|s| self.symbol_of(&s)),
            ImportDirective {
                symbol,
                qualified_symbol,
                ..
            } => {
                if symbol != Id::NULL {
                    self.find_node(symbol).and_then(|s| self.symbol_of(&s))
                } else {
                    let qualified_symbol = self.find_node(qualified_symbol)?;
                    if let QualifiedSymbol { symbols, .. } = qualified_symbol.kind {
                        self.find_node(*symbols.last()?)
                            .and_then(|s| self.symbol_of(&s))
                    } else {
                        None
                    }
                }
            }
            _ => None,
        }
    }

    pub fn find_usage(
        &self,
        node: &Node,
        kind: DeclarationKind,
        types: &Types,
    ) -> Option<Arc<Usage>> {
        self.usage_cache.gate(&(kind.clone(), node.id), || {
            if node.is_declaration(kind) {
                Some(Arc::new(semantics::Usage {
                    declaration: node.clone(),
                    references: self.find_references(node, kind),
                    import_directives: self.find_import_directives_from_declaration(node),
                }))
            } else if node.is_import_directive() {
                let declaration = &self.find_declaration_from_import(&node)?;
                self.find_usage(declaration, declaration.declaration_kind(), types)
            } else if node.is_operator() {
                let usage_target = &self.usage_target_from_operator(node)?;
                self.find_usage(usage_target, usage_target.declaration_kind(), types)
            } else if node.is_symbol() {
                let usage_target = &self.usage_target_from_symbol(node)?;
                self.find_usage(usage_target, usage_target.declaration_kind(), types)
            } else if node.is_reference(kind) {
                let declaration = &self.find_declaration(node, node.declaration_kind())?;
                self.find_usage(declaration, declaration.declaration_kind(), types)
            } else if node.is_method() {
                Some(Arc::new(semantics::Usage {
                    declaration: node.clone(),
                    references: self.find_method_references(node, types),
                    import_directives: vec![],
                }))
            } else if node.is_message_pattern() {
                let parent = self.parent(node)?;
                match parent.kind {
                    Signature { .. } => {
                        let method = self.parent(&parent)?;
                        self.find_usage(&method, DeclarationKind::None, types)
                    }
                    Initializer { .. } => self.find_usage(&parent, DeclarationKind::None, types),
                    _ => None,
                }
            } else if node.is_message() {
                let method = self.method_from_message(node, types)?;
                self.find_usage(&method, DeclarationKind::None, types)
            } else if node.is_initializer() {
                Some(Arc::new(semantics::Usage {
                    declaration: node.clone(),
                    references: self.find_method_references(node, types),
                    import_directives: vec![],
                }))
            } else {
                None
            }
        })
    }

    pub fn method_from_message(&self, message: &Node, types: &Types) -> Option<Node> {
        let send = self.parent(message)?;
        if let MessageSendExpression { expression, .. } = send.kind {
            let receiver = self.find_child(&send, expression)?;
            let type_ = types.get_type_of_expression(&receiver);
            let behaviours = types.get_behaviours(&type_);

            let selector = self.message_selector(message)?;
            for behaviour in behaviours {
                if behaviour.selector() == selector {
                    return self.find_node(behaviour.method_id);
                }
            }
        }
        None
    }

    pub fn find_method_references(&self, method: &Node, types: &Types) -> Vec<Node> {
        let mut messages = vec![];
        for message in self.all_messages() {
            if let Some(matching_method) = self.method_from_message(&message, types) {
                if matching_method.id == method.id {
                    messages.push(message);
                }
            }
        }
        messages
    }

    pub fn find_import_directives_from_declaration(&self, declaration: &Node) -> Vec<Node> {
        let mut imports = vec![];

        if let Some((Some(module_id), syntax::Exported(_, _, _))) =
            self.parent(declaration).map(|n| (n.parent_id, n.kind))
        {
            if let Some(module) = self.find_node_in(&declaration.span.start.uri, module_id) {
                if let Some((namespace, _)) = self.namespace_of_module(&module) {
                    if let Some((name, _)) = self.symbol_of(declaration) {
                        let qualified_exported_name = format!("{}/{}", namespace, name);

                        imports.extend(self.imports_matching(qualified_exported_name));
                    }
                }
            }
        }

        imports
    }

    pub fn imports_matching(&self, qualified_name: String) -> Vec<Node> {
        self.all_imports()
            .into_iter()
            .filter(|import| {
                if let ImportDirective {
                    qualified_symbol, ..
                } = import.kind
                {
                    if let Some(qualified_symbol) = self.find_child(import, qualified_symbol) {
                        if self.qualified_symbol_to_string(&qualified_symbol) == qualified_name {
                            return true;
                        }
                    }
                }
                false
            })
            .collect()
    }

    pub fn all_message_sends(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_message_send())
    }

    pub fn all_type_parameters(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_type_parameter())
    }

    pub fn all_messages(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_message())
    }

    pub fn all_classes(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_class())
    }

    pub fn all_initializers(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_initializer())
    }

    pub fn all_is_directives(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_is_directive())
    }

    pub fn all_imports(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_import_directive())
    }

    pub fn usage_target_from_operator(&self, operator: &Node) -> Option<Node> {
        let parent = self.parent(operator)?;

        if parent.is_message_pattern() || parent.is_message() {
            return Some(parent);
        }

        None
    }

    pub fn namespace_of_uri(&self, uri: &URI) -> Option<(String, Node)> {
        let module = self.modules.get(uri)?.root()?;
        self.namespace_of_module(module)
    }

    pub fn qualified_name_of(&self, declaration: &Node) -> Option<(String, Option<Node>, Node)> {
        let (name, name_node) = self.symbol_of(&declaration)?;

        match self.namespace_of_uri(&declaration.span.start.uri) {
            None => Some((name, None, name_node)),
            Some((namespace, namespace_node)) => Some((
                format!("{}/{}", namespace, name),
                Some(namespace_node),
                name_node,
            )),
        }
    }

    pub fn qualified_name_of_method(&self, method: &Node) -> Option<String> {
        let selector = self.method_selector(method)?;
        let class = self.closest_class_upwards(method)?;

        let (qn, _, _) = self.qualified_name_of(&class)?;

        Some(format!("{}#{}", qn, selector))
    }

    pub fn usage_target_from_symbol(&self, symbol: &Node) -> Option<Node> {
        let parent = self.parent(symbol)?;
        if parent.is_qualified_symbol() {
            if let Some(qs_parent) = self.parent(&parent) {
                if qs_parent.is_import_directive() {
                    return Some(qs_parent);
                }
            }
        }

        if parent.is_declaration(DeclarationKind::Any)
            || parent.is_reference(DeclarationKind::Any)
            || parent.is_import_directive()
        {
            return Some(parent);
        }

        if parent.is_message() {
            return Some(parent);
        }

        if let KeywordPair { .. } = parent.kind {
            let parent = self.parent(&parent)?;

            if parent.is_message() {
                return Some(parent);
            }
        }

        if parent.is_message_pattern() {
            return Some(parent);
        }

        if let KeywordPair { .. } = parent.kind {
            let parent = self.parent(&parent)?;

            if parent.is_message_pattern() {
                return Some(parent);
            }
        }

        None
    }

    pub fn find_child(&self, parent: &Node, child_id: Id) -> Option<Node> {
        self.find_node_in(&parent.span.start.uri, child_id)
    }

    pub fn find_references(&self, declaration: &Node, kind: DeclarationKind) -> Vec<Node> {
        let mut references = vec![];

        if let Some((name, _)) = self.symbol_of(declaration) {
            // If the declaration is exported
            if let Some((module_id, syntax::Exported(_, _, _))) =
                self.parent(declaration).map(|n| (n.parent_id, n.kind))
            {
                if let Some(refs) = module_id
                    .and_then(|id| self.find_node(id))
                    .map(|m| self.find_references_through_imports(m, name.clone(), kind))
                {
                    references.extend(refs);
                }
            }

            let mut start_scope_root_search_from = declaration.clone();

            if declaration.is_scope_root() {
                if let Some(parent) = self.parent(declaration) {
                    start_scope_root_search_from = parent;
                }
            }

            match self.closest_scope_root_upwards(&start_scope_root_search_from) {
                None => (),
                Some(scope_root) => references.extend(self.all_downwards(&scope_root, &|n| {
                    if !n.is_reference(kind) {
                        return false;
                    }

                    if let Some(dec) = self.find_declaration(n, kind) {
                        if declaration.is_import_directive() {
                            // Reference is referencing an import
                            if dec.span.start.uri != n.span.start.uri {
                                if let Some((ref_name, _)) = self.symbol_of(n) {
                                    if ref_name == name {
                                        return true;
                                    }
                                }
                            }
                        }

                        if dec.id == declaration.id {
                            return true;
                        }
                    }

                    return false;
                })),
            }
        }

        references
    }

    pub fn find_declaration(&self, reference: &Node, kind: DeclarationKind) -> Option<Node> {
        match reference.kind {
            SelfExpression(_) | SelfTypeExpression(_) => self.closest_class_upwards(reference),
            _ => {
                let (name, _) = self.symbol_of(reference)?;
                match self.find_declaration_above(reference, name.clone(), kind) {
                    Some(t) => Some(t),
                    None => self.find_stdlib_class(format!("Loa/{}", name).as_str()),
                }
            }
        }
    }

    pub fn find_declaration_above(
        &self,
        node: &Node,
        name: String,
        kind: DeclarationKind,
    ) -> Option<Node> {
        let scope_root = self.closest_scope_root_upwards(node)?;
        let mut result = None;
        let mut traverse = |node: &Node| {
            if node.is_import_directive() {
                if let syntax::ImportDirective {
                    qualified_symbol,
                    symbol,
                    ..
                } = node.kind
                {
                    if let Some(qualified_symbol) = self.find_child(node, qualified_symbol) {
                        if let syntax::QualifiedSymbol { ref symbols, .. } = qualified_symbol.kind {
                            if let Some(mut imported_symbol) = symbols.last().cloned() {
                                if symbol != Id::NULL {
                                    imported_symbol = symbol;
                                }
                                if let Some(imported_symbol) =
                                    self.find_child(&qualified_symbol, imported_symbol)
                                {
                                    if let syntax::Symbol(t) = imported_symbol.kind {
                                        if t.lexeme() == name {
                                            match self.find_declaration_from_import(node) {
                                                Some(n) => {
                                                    result = Some(n);
                                                }
                                                None => {
                                                    result = Some(node.clone());
                                                }
                                            }
                                            return false;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if node.is_declaration(kind) {
                if let Some((n, _)) = self.symbol_of(node) {
                    if n == name {
                        result = Some(node.clone());
                        return false;
                    }
                }
            }

            // We do not traverse down scope roots, since
            // declarations declared there is not reachable
            // to the original reference.
            node.id == scope_root.id || !node.is_scope_root() || node.is_repl_line()
        };

        if scope_root.is_repl_line() {
            self.traverse_all_repl_lines(&mut traverse);
        } else {
            self.traverse(&scope_root, &mut traverse);
        }

        if result.is_some() {
            return result;
        }
        let parent = self.parent(&scope_root)?;
        self.find_declaration_above(&parent, name, kind)
    }

    pub fn find_references_through_imports(
        &self,
        module: syntax::Node,
        exported_name: String,
        kind: DeclarationKind,
    ) -> Vec<syntax::Node> {
        let mut references = vec![];
        if let Some((namespace, _)) = self.namespace_of_module(&module) {
            let name = format!("{}/{}", namespace, exported_name);
            for module in self.modules() {
                for import_directive in self.import_directives_of_module(&module) {
                    if let ImportDirective {
                        qualified_symbol, ..
                    } = import_directive.kind
                    {
                        if let Some(qs) = self.find_child(&import_directive, qualified_symbol) {
                            if self.qualified_symbol_to_string(&qs) == name {
                                references.extend(self.find_references(&import_directive, kind));
                            }
                        }
                    }
                }
            }
        }
        references
    }

    pub fn import_directives_of_module(&self, module: &Node) -> Vec<Node> {
        self.all_downwards(module, &|n| n.is_import_directive())
    }

    pub fn find_declaration_from_import(&self, import_directive: &Node) -> Option<Node> {
        if let ImportDirective {
            qualified_symbol, ..
        } = import_directive.kind
        {
            let qs = self.find_child(import_directive, qualified_symbol)?;
            if let QualifiedSymbol { ref symbols, .. } = qs.kind {
                let mut symbols = symbols
                    .iter()
                    .filter_map(|sid| self.find_child(&qs, *sid))
                    .filter_map(|symbol| self.symbol_of(&symbol))
                    .map(|(s, _)| s)
                    .collect::<Vec<_>>();
                let imported_symbol = symbols.pop()?;
                let imported_namespace = symbols.join("/");

                for module in self.modules_in_namespace(imported_namespace) {
                    for (_, declaration) in self.module_declarations_in(&module) {
                        // TODO: Make checker that makes sure that imports are importing exports
                        if let Some((s, _)) = self.symbol_of(&declaration) {
                            if s == imported_symbol {
                                return Some(declaration);
                            }
                        }
                    }
                }
            }
        }
        None
    }

    pub fn modules_in_namespace(&self, namespace: String) -> Vec<Node> {
        self.modules()
            .into_iter()
            .filter(|module| match self.namespace_of_module(module) {
                Some((n, _)) if n == namespace => true,
                _ => false,
            })
            .collect()
    }

    pub fn namespace_of_module(&self, module: &Node) -> Option<(String, Node)> {
        if let Module {
            namespace_directive,
            ..
        } = module.kind
        {
            if let NamespaceDirective {
                qualified_symbol, ..
            } = self.find_child(module, namespace_directive)?.kind
            {
                let qs = self.find_child(module, qualified_symbol)?;
                return Some((self.qualified_symbol_to_string(&qs), qs));
            }
        }
        None
    }

    pub fn qualified_symbol_to_string(&self, qualified_symbol: &Node) -> String {
        self.qualified_symbol_to_strings(qualified_symbol).join("/")
    }

    pub fn qualified_symbol_to_strings(&self, qualified_symbol: &Node) -> Vec<String> {
        if let QualifiedSymbol { ref symbols, .. } = qualified_symbol.kind {
            symbols
                .iter()
                .filter_map(|s| self.find_child(qualified_symbol, *s))
                .filter_map(|s| self.symbol_to_string(&s))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn symbol_to_string(&self, symbol: &Node) -> Option<String> {
        if let Symbol(ref t) = symbol.kind {
            Some(t.lexeme())
        } else {
            None
        }
    }

    pub fn operator_to_string(&self, symbol: &Node) -> Option<String> {
        if let Operator(ref t) = symbol.kind {
            Some(t.iter().map(Token::lexeme).collect())
        } else {
            None
        }
    }

    pub fn module_declarations_in(&self, module: &Node) -> Vec<(bool, Node)> {
        if let Module {
            ref module_declarations,
            ..
        } = module.kind
        {
            module_declarations
                .iter()
                .filter_map(|mdi| self.find_child(module, *mdi))
                .filter_map(|module_declaration| {
                    if let Exported(_, _, d) = module_declaration.kind {
                        Some((true, self.find_child(module, d)?))
                    } else {
                        Some((false, module_declaration))
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    pub fn super_type_expressions(&self, class: &Node) -> Vec<Node> {
        let mut super_type_expressions = vec![];

        if let Class { class_body, .. } = class.kind {
            if let Some(class_body) = self.find_child(class, class_body) {
                if let ClassBody {
                    ref class_members, ..
                } = class_body.kind
                {
                    for class_member in class_members.iter() {
                        if let Some(class_member) = self.find_child(&class_body, *class_member) {
                            if let IsDirective {
                                type_expression, ..
                            } = class_member.kind
                            {
                                if let Some(type_expression) =
                                    self.find_child(&class_member, type_expression)
                                {
                                    super_type_expressions.push(type_expression);
                                }
                            }
                        }
                    }
                }
            }
        }

        super_type_expressions
    }

    pub fn all_expressions(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_expression())
    }

    pub fn all_references(&self, kind: DeclarationKind) -> Vec<Node> {
        self.all_matching(|n| n.is_reference(kind))
    }

    pub fn all_reference_symbols(&self, kind: DeclarationKind) -> Vec<Node> {
        self.all_references(kind)
            .into_iter()
            .filter_map(|reference| self.symbol_of(&reference))
            .map(|(_, n)| n)
            .collect()
    }

    pub fn all_matching<F: Fn(&Node) -> bool>(&self, f: F) -> Vec<Node> {
        cache_candidate("all_matching", || {
            let mut matching = vec![];

            self.traverse_all(&mut |n| {
                if f(n) {
                    matching.push(n.clone());
                }
                true
            });

            matching
        })
    }

    pub fn child_nodes(&self, node: &Node) -> Vec<Node> {
        let uri = &node.span.start.uri;
        let mut out = vec![];
        for child_id in node.children() {
            if let Some(n) = self.find_node_in(uri, child_id) {
                out.push(n);
            }
        }
        out
    }

    /// Traverses all nodes in the tree below this point.
    /// If the callback returns true for a given node, the
    /// traversal will continue down its children. Otherwise,
    /// the traversal will not traverse down that path.
    pub fn traverse<F: FnMut(&Node) -> bool>(&self, from: &Node, f: &mut F) {
        if !f(from) {
            return;
        }

        for child in self.child_nodes(from) {
            self.traverse(&child, f);
        }
    }

    pub fn closest_upwards<F: Fn(&Node) -> bool>(&self, node: &Node, f: F) -> Option<Node> {
        if f(node) {
            return Some(node.clone());
        }
        let uri = &node.span.start.uri;
        let mut parent = node.parent_id?;
        loop {
            let parent_node = self.find_node_in(uri, parent)?;
            if f(&parent_node) {
                return Some(parent_node.clone());
            }
            for child in self.child_nodes(&parent_node) {
                if f(&child) {
                    return Some(child.clone());
                }
            }
            parent = parent_node.parent_id?;
        }
    }

    pub fn all_downwards<F: Fn(&Node) -> bool>(&self, from: &Node, f: &F) -> Vec<Node> {
        let mut nodes = vec![];

        if f(from) {
            nodes.push(from.clone());
        }

        for child in self.child_nodes(from) {
            nodes.extend(self.all_downwards(&child, f));
        }

        nodes
    }

    pub fn any_downwards<F: Fn(&Node) -> bool>(&self, from: &Node, f: &F) -> bool {
        if f(from) {
            return true;
        }

        for child in self.child_nodes(from) {
            if self.any_downwards(&child, f) {
                return true;
            }
        }

        false
    }

    pub fn closest_expression_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_expression())
    }

    pub fn closest_message_send_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_message_send())
    }

    pub fn closest_message_pattern_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_message_pattern())
    }

    pub fn all_expressions_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_expression())
    }

    pub fn closest_type_expression_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_type_expression())
    }

    pub fn all_type_expressions_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_type_expression())
    }

    pub fn closest_class_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_class())
    }

    pub fn closest_scope_root_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_scope_root())
    }

    pub fn all_scope_roots_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_scope_root())
    }

    pub fn all_scope_roots(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_scope_root())
    }

    pub fn all_reference_type_expressions(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_reference_type_expression())
    }

    pub fn all_number_literals(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_number_literal())
    }

    pub fn closest_declaration_upwards(&self, from: &Node, kind: DeclarationKind) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_declaration(kind))
    }

    pub fn all_declarations_downwards(&self, from: &Node, kind: DeclarationKind) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_declaration(kind))
    }

    pub fn all_classes_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_class())
    }

    pub fn closest_is_directive_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_is_directive())
    }

    pub fn all_is_directives_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_is_directive())
    }

    pub fn closest_references_upwards(&self, from: &Node, kind: DeclarationKind) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_reference(kind))
    }

    pub fn all_references_downwards(&self, from: &Node, kind: DeclarationKind) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_reference(kind))
    }

    pub fn all_declarations_in_scope(&self, scope_root: &Node, kind: DeclarationKind) -> Vec<Node> {
        let mut declarations = vec![];
        self.traverse(scope_root, &mut |n| {
            if n.is_declaration(kind) || n.is_import_directive() {
                declarations.push(n.clone());
            }

            !n.is_scope_root() || n.id == scope_root.id
        });
        declarations
    }

    pub fn declaration_is_exported(&self, declaration: &Node) -> bool {
        if let Some(parent) = self.parent(declaration) {
            if let Exported(_, _, _) = parent.kind {
                return true;
            }
        }
        false
    }

    pub fn message_pattern_of_method(&self, method: &Node) -> Option<Node> {
        if let Method { signature, .. } = method.kind {
            let signature = self.find_child(method, signature)?;
            if let Signature {
                message_pattern, ..
            } = signature.kind
            {
                return self.find_child(&signature, message_pattern);
            }
        }
        None
    }

    pub fn message_pattern_of_initializer(&self, initializer: &Node) -> Option<Node> {
        if let Initializer { message_pattern, .. } = initializer.kind {
            return self.find_child(initializer, message_pattern);
        }
        None
    }

    pub fn message_pattern_selector(&self, message_pattern: &Node) -> Option<String> {
        match message_pattern.kind {
            UnaryMessagePattern { symbol, .. } => {
                let symbol = self.find_child(message_pattern, symbol)?;
                if let Symbol(ref t) = symbol.kind {
                    return Some(t.lexeme());
                }
            }

            BinaryMessagePattern { operator, .. } => {
                let operator = self.find_child(message_pattern, operator)?;
                if let Operator(ref ts) = operator.kind {
                    return Some(ts.iter().map(|t| t.lexeme()).collect::<Vec<_>>().join(""));
                }
            }

            KeywordMessagePattern {
                ref keyword_pairs, ..
            } => {
                let mut selector = String::new();

                for pair in keyword_pairs.iter() {
                    let pair = self.find_child(message_pattern, *pair)?;
                    if let KeywordPair { keyword, .. } = pair.kind {
                        let keyword = self.find_child(&pair, keyword)?;
                        if let Symbol(ref t) = keyword.kind {
                            selector.push_str(format!("{}:", t.lexeme()).as_ref());
                        }
                    }
                }

                if selector.len() > 0 {
                    return Some(selector);
                }
            }

            _ => (),
        }
        None
    }

    pub fn root_of(&self, uri: &URI) -> Option<Node> {
        let tree = self.modules.get(uri)?;
        tree.root().cloned()
    }

    pub fn methods_of_class(&self, class: &Node) -> Vec<Node> {
        let mut methods = vec![];

        if let Class { class_body, .. } = class.kind {
            if let Some(class_body) = self.find_child(class, class_body) {
                if let ClassBody {
                    ref class_members, ..
                } = class_body.kind
                {
                    for class_member in class_members.iter() {
                        if let Some(class_member) = self.find_child(&class_body, *class_member) {
                            if let Method { .. } = class_member.kind {
                                methods.push(class_member);
                            }
                        }
                    }
                }
            }
        }

        methods
    }

    pub fn variables_of_class(&self, class: &Node) -> Vec<Node> {
        let mut variables = vec![];

        if let Class { class_body, .. } = class.kind {
            if let Some(class_body) = self.find_child(class, class_body) {
                if let ClassBody {
                    ref class_members, ..
                } = class_body.kind
                {
                    for class_member in class_members.iter() {
                        if let Some(class_member) = self.find_child(&class_body, *class_member) {
                            if let Variable { .. } = class_member.kind {
                                variables.push(class_member);
                            }
                        }
                    }
                }
            }
        }

        variables
    }

    pub fn message_arity(&self, message: &Node) -> Option<usize> {
        match message.kind {
            UnaryMessage { .. } => Some(1),
            BinaryMessage { .. } => Some(2),
            KeywordMessage {
                ref keyword_pairs, ..
            } => Some(keyword_pairs.len() + 1),
            _ => None,
        }
    }

    pub fn message_pattern_arity(&self, message_pattern: &Node) -> Option<usize> {
        match message_pattern.kind {
            UnaryMessagePattern { .. } => Some(1),
            BinaryMessagePattern { .. } => Some(2),
            KeywordMessagePattern {
                ref keyword_pairs, ..
            } => Some(keyword_pairs.len() + 1),
            _ => None,
        }
    }

    pub fn method_arity(&self, method: &Node) -> Option<usize> {
        if let Method { signature, .. } = method.kind {
            let signature = self.find_child(method, signature)?;
            if let Signature {
                message_pattern, ..
            } = signature.kind
            {
                let message_pattern = self.find_child(&signature, message_pattern)?;

                match message_pattern.kind {
                    UnaryMessagePattern { .. } => return Some(1),
                    BinaryMessagePattern { .. } => return Some(2),
                    KeywordMessagePattern { keyword_pairs, .. } => {
                        return Some(keyword_pairs.len() + 1)
                    }
                    _ => {}
                }
            }
        }
        None
    }

    pub fn index_of_parameter(&self, parameter: &Node) -> Option<usize> {
        let parent = self.parent(parameter)?;

        if let BinaryMessagePattern { .. } = parent.kind {
            return Some(0);
        }

        if let KeywordPair { .. } = parent.kind {
            let pattern = self.parent(&parent)?;

            if let KeywordMessagePattern { keyword_pairs, .. } = pattern.kind {
                for (index, pair_id) in keyword_pairs.iter().enumerate() {
                    if *pair_id == parent.id {
                        return Some(index);
                    }
                }
            }
        }

        None
    }

    pub fn all_stdlib_classes(&self) -> HashMap<String, Node> {
        self.stdlib_classes_cache.gate(&(), || {
            let mut classes = HashMap::new();
            for (u, t) in self.modules.iter() {
                if u.is_stdlib() {
                    if let Some(root) = t.root() {
                        for class in self.all_classes_downwards(root) {
                            if let Some((qn, _, _)) = self.qualified_name_of(&class) {
                                classes.insert(qn, class);
                            }
                        }
                    }
                }
            }
            classes
        })
    }

    pub fn find_stdlib_class(&self, name: &str) -> Option<Node> {
        self.stdlib_class_cache.gate(&name.into(), || {
            for (u, t) in self.modules.iter() {
                if u.is_stdlib() {
                    if let Some(root) = t.root() {
                        for class in self.all_classes_downwards(root) {
                            if let Some((qn, _, _)) = self.qualified_name_of(&class) {
                                if qn == name {
                                    return Some(class);
                                }
                            }
                        }
                    }
                }
            }
            None
        })
    }

    pub fn type_arguments_of_type_argument_list(&self, type_argument_list: &Node) -> Vec<Node> {
        if let TypeArgumentList {
            ref type_expressions,
            ..
        } = type_argument_list.kind
        {
            type_expressions
                .iter()
                .filter_map(|i| self.find_child(type_argument_list, *i))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn type_arguments_of_reference_type_expression(&self, reference: &Node) -> Vec<Node> {
        if let ReferenceTypeExpression {
            type_argument_list, ..
        } = reference.kind
        {
            if let Some(type_argument_list) = self.find_child(reference, type_argument_list) {
                return self.type_arguments_of_type_argument_list(&type_argument_list);
            }
        }
        vec![]
    }

    pub fn type_parameters_of_type_parameter_list(&self, type_parameter_list: &Node) -> Vec<Node> {
        if let TypeParameterList {
            ref type_parameters,
            ..
        } = type_parameter_list.kind
        {
            type_parameters
                .iter()
                .filter_map(|i| self.find_child(type_parameter_list, *i))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn type_parameters_of_type_declaration(&self, declaration: &Node) -> Vec<Node> {
        if let Class {
            type_parameter_list,
            ..
        } = declaration.kind
        {
            if let Some(type_parameter_list) = self.find_child(declaration, type_parameter_list) {
                return self.type_parameters_of_type_parameter_list(&type_parameter_list);
            }
        }
        vec![]
    }

    pub fn method_selector(&self, method: &Node) -> Option<String> {
        if let Method { signature, .. } = method.kind {
            let signature = self.find_child(method, signature)?;
            if let Signature {
                message_pattern, ..
            } = signature.kind
            {
                let message_pattern = self.find_child(&signature, message_pattern)?;
                return self.message_pattern_selector(&message_pattern);
            }
        }
        None
    }

    pub fn all_super_classes_of(&self, class: &Node) -> Vec<Node> {
        let mut super_classes = vec![];
        self.collect_super_classes_of(class, &mut super_classes);
        super_classes
    }

    fn collect_super_classes_of(&self, class: &Node, super_classes: &mut Vec<Node>) {
        for is_directive in self.all_is_directives_downwards(class) {
            if let IsDirective {
                type_expression, ..
            } = is_directive.kind
            {
                if let Some(type_expression) = self.find_child(&is_directive, type_expression) {
                    if let Some(super_class) =
                        self.find_declaration(&type_expression, DeclarationKind::Type)
                    {
                        if super_class.is_class() {
                            self.collect_super_classes_of(&super_class, super_classes);
                            super_classes.push(super_class);
                        }
                    }
                }
            }
        }
    }

    pub fn all_sub_classes_of(&self, class: &Node) -> Vec<Node> {
        let mut sub_classes = vec![];
        self.collect_sub_classes_of(class, &mut sub_classes);
        sub_classes
    }

    fn collect_sub_classes_of(&self, class: &Node, sub_classes: &mut Vec<Node>) {
        for is_directive in self.all_is_directives() {
            if let IsDirective {
                type_expression, ..
            } = is_directive.kind
            {
                if let Some(type_expression) = self.find_child(&is_directive, type_expression) {
                    if let Some(declaration) =
                        self.find_declaration(&type_expression, DeclarationKind::Type)
                    {
                        if declaration.id == class.id {
                            if let Some(sub_class) = self.closest_class_upwards(&is_directive) {
                                self.collect_sub_classes_of(&sub_class, sub_classes);
                                sub_classes.push(sub_class);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn tree_of(&self, node: &Node) -> Option<&Arc<Tree>> {
        self.modules.get(&node.span.start.uri)
    }

    pub fn doc_of(&self, node: &Node) -> Option<Node> {
        if let Class { doc, .. } | Method { doc, .. } = node.kind {
            if let Some(doc) = self.find_child(node, doc) {
                return Some(doc);
            }
        }
        if let Exported(doc, _, _) = self.parent(node)?.kind {
            return self.find_child(node, doc);
        }
        None
    }

    pub fn blocks_of_doc(&self, doc: &Node) -> Vec<Node> {
        let mut result = vec![];
        if let Doc { ref blocks, .. } = doc.kind {
            for block in blocks {
                if let Some(block) = self.find_child(doc, *block) {
                    result.push(block);
                }
            }
        }
        result
    }

    pub fn visibility_of_method(&self, method: &Node) -> Option<Token> {
        if let Method { ref visibility, .. } = method.kind {
            visibility.clone()
        } else {
            None
        }
    }

    pub fn method_is_native(&self, method: &Node) -> bool {
        if let Method {
            ref native_keyword, ..
        } = method.kind
        {
            native_keyword.is_some()
        } else {
            false
        }
    }

    pub fn method_is_visible_from(&self, method: &Node, source_node: &Node) -> Option<bool> {
        if self.visibility_of_method(method)?.kind != TokenKind::PrivateKeyword {
            return Some(true);
        }

        let method_class = self.closest_class_upwards(method)?;
        let source_class = self.closest_class_upwards(source_node);

        Some(source_class.is_some() && method_class.id == source_class?.id)
    }

    pub fn declarations_in_scope(
        &self,
        mut from: syntax::Node,
        kind: DeclarationKind,
    ) -> Vec<(String, syntax::Node)> {
        let uri = from.span.start.uri.clone();

        let mut declarations = HashMap::new();

        while let Some(scope_root) = self.closest_scope_root_upwards(&from) {
            let mut traverse = |n: &syntax::Node| {
                if n.is_declaration(kind) {
                    if let Some((name, _)) = self.symbol_of(&n) {
                        declarations.insert(name, n.clone());
                    }
                }

                if n.is_import_directive() {
                    if let Some((name, _)) = self.symbol_of(&n) {
                        if let Some(n) = self.find_declaration_from_import(&n) {
                            declarations.insert(name, n.clone());
                        }
                    }
                }

                // Don't traverse into lower scopes.
                n.id == scope_root.id || !n.is_scope_root() || n.is_repl_line()
            };
            if scope_root.is_repl_line() {
                self.traverse_all_repl_lines(&mut traverse);
            } else {
                self.traverse(&scope_root, &mut traverse);
            }

            if let Some(parent) = scope_root
                .parent_id
                .and_then(|pid| self.find_node_in(&uri, pid))
            {
                from = parent;
            } else {
                break;
            }
        }

        for (_, class) in self.all_stdlib_classes() {
            if let Some((name, _)) = self.symbol_of(&class) {
                if !declarations.contains_key(&name) {
                    declarations.insert(name, class);
                }
            }
        }

        declarations.into_iter().collect()
    }

    pub fn method_body(&self, method: &Node) -> Option<Node> {
        if let Method { method_body, .. } = method.kind {
            let method_body = self.find_child(method, method_body)?;
            if let MethodBody { expression, .. } = method_body.kind {
                return self.find_child(&method_body, expression);
            }
        }
        None
    }

    pub fn keyword_pair(&self, keyword_pair: &Node) -> Option<(Node, Node)> {
        if let KeywordPair { keyword, value, .. } = keyword_pair.kind {
            if let Some(keyword) = self.find_child(keyword_pair, keyword) {
                if let Some(value) = self.find_child(keyword_pair, value) {
                    return Some((keyword, value));
                }
            }
        }
        None
    }

    pub fn keyword_pairs(&self, parent: &Node, keyword_pair_ids: &Vec<Id>) -> Vec<(Node, Node)> {
        keyword_pair_ids
            .iter()
            .filter_map(|i| self.find_child(parent, *i))
            .filter_map(|pair| self.keyword_pair(&pair))
            .collect()
    }

    pub fn message_pattern_parameters(&self, message_pattern: &Node) -> Vec<Node> {
        match message_pattern.kind {
            BinaryMessagePattern {
                parameter_pattern, ..
            } => self
                .find_child(message_pattern, parameter_pattern)
                .into_iter()
                .collect(),
            KeywordMessagePattern {
                ref keyword_pairs, ..
            } => self
                .keyword_pairs(message_pattern, keyword_pairs)
                .into_iter()
                .map(|(_, v)| v)
                .collect(),
            _ => vec![],
        }
    }

    pub fn signature_parameters(&self, signature: &Node) -> Vec<Node> {
        if let Signature {
            message_pattern, ..
        } = signature.kind
        {
            if let Some(message_pattern) = self.find_child(signature, message_pattern) {
                return self.message_pattern_parameters(&message_pattern);
            }
        }

        vec![]
    }

    pub fn method_parameters(&self, method: &Node) -> Vec<Node> {
        if let Method { signature, .. } = method.kind {
            if let Some(signature) = self.find_child(method, signature) {
                return self.signature_parameters(&signature);
            }
        }

        vec![]
    }

    pub fn methods_overridden_by(&self, method: &Node) -> Vec<Node> {
        self.methods_overridden_by_impl(method).unwrap_or(vec![])
    }

    fn methods_overridden_by_impl(&self, method: &Node) -> Option<Vec<Node>> {
        let selector = self.method_selector(method)?;
        let class = self.closest_class_upwards(method)?;

        let mut methods = vec![];

        for super_type in self.super_type_expressions(&class) {
            if let Some(super_class) = self.find_declaration(&super_type, DeclarationKind::Type) {
                for super_method in self.methods_of_class(&super_class) {
                    if let Some(super_method_selector) = self.method_selector(&super_method) {
                        if super_method_selector == selector {
                            methods.extend(self.methods_overridden_by(&super_method));

                            methods.push(super_method);
                        }
                    }
                }
            }
        }

        Some(methods)
    }

    pub fn locals_crossing_into(&self, expression: &Node) -> Vec<Node> {
        let locals: HashMap<_, _> = self
            .all_references_downwards(expression, DeclarationKind::Value)
            .into_iter()
            .filter_map(|r| self.find_declaration(&r, DeclarationKind::Value))
            .filter(|d| !matches!(d.kind, Class{..}))
            .filter(|d| !self.is_within(&d, expression))
            .map(|d| (d.id, d))
            .collect();

        let mut locals: Vec<_> = locals.into_iter().map(|(_, d)| d).collect();

        locals.sort_by(|a, b| a.span.start.offset.cmp(&b.span.start.offset));

        locals
    }

    pub fn self_crosses_into(&self, expression: &Node) -> bool {
        self.any_downwards(expression, &|n| matches!(n.kind, SelfExpression(_)))
    }

    pub fn is_within(&self, needle: &Node, haystack: &Node) -> bool {
        self.any_downwards(haystack, &|n| n.id == needle.id)
    }

    /// Whether a class has a class object (i.e. isn't a constant class).
    pub fn has_class_object(&self, class: &Node) -> bool {
        self.has_class_object_impl(class).unwrap_or(false)
    }

    fn has_class_object_impl(&self, class: &Node) -> Option<bool> {
        if let Class { class_body, .. } = class.kind {
            let class_body = self.find_child(class, class_body)?;
            if let ClassBody { class_members, .. } = class_body.kind {
                for member in class_members {
                    if let Some(member) = self.find_child(class, member) {
                        match member.kind {
                            Initializer { .. } => return Some(true),
                            _ => {}
                        }
                    }
                }
            }
        }
        None
    }

    pub fn initializers_of(&self, class: &Node) -> Vec<Node> {
        let mut initializers = vec![];
        if let Class { class_body, .. } = class.kind {
            if let Some(class_body) = self.find_child(class, class_body) {
                if let ClassBody { class_members, .. } = class_body.kind {
                    for member in class_members {
                        if let Some(member) = self.find_child(class, member) {
                            match member.kind {
                                Initializer { .. } => initializers.push(member),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        initializers
    }

    pub fn initializer_selector(&self, initializer: &Node) -> Option<String> {
        if let Initializer {
            message_pattern, ..
        } = initializer.kind
        {
            let message_pattern = self.find_child(initializer, message_pattern)?;
            self.message_pattern_selector(&message_pattern)
        } else {
            None
        }
    }

    pub fn initializer_arity(&self, initializer: &Node) -> Option<usize> {
        if let Initializer {
            message_pattern, ..
        } = initializer.kind
        {
            let message_pattern = self.find_child(initializer, message_pattern)?;
            self.message_pattern_arity(&message_pattern)
        } else {
            None
        }
    }

    pub fn initializer_parameters(&self, initializer: &Node) -> Vec<Node> {
        if let Initializer {
            message_pattern, ..
        } = initializer.kind
        {
            if let Some(message_pattern) = self.find_child(initializer, message_pattern) {
                self.message_pattern_parameters(&message_pattern)
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    pub fn initializer_assignments(&self, initializer: &Node) -> Vec<(String, Node)> {
        if let Initializer {
            ref keyword_pairs, ..
        } = initializer.kind
        {
            self.keyword_pairs(initializer, keyword_pairs)
                .into_iter()
                .filter_map(|(k, v)| Some((self.symbol_of(&k)?.0, v)))
                .collect()
        } else {
            vec![]
        }
    }
}
