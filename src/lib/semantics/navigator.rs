use crate::semantics::*;
use crate::syntax::*;
use crate::*;

pub trait Navigator
where
    Self: Clone,
{
    fn traverse_all<F: FnMut(&Node) -> bool>(&self, f: &mut F);
    fn modules(&self) -> Vec<Node>;
    fn find_node(&self, id: Id) -> Option<Node>;

    fn find_node_in(&self, _uri: &URI, id: Id) -> Option<Node> {
        self.find_node(id)
    }

    fn parent(&self, child: &Node) -> Option<Node> {
        let uri = &child.span.start.uri;
        child.parent_id.and_then(|i| self.find_node_in(uri, i))
    }

    fn children(&self, parent: &Node) -> Vec<Node> {
        let uri = &parent.span.start.uri;
        parent
            .kind
            .children()
            .into_iter()
            .filter_map(|i| self.find_node_in(uri, i))
            .collect()
    }

    fn message_selector(&self, message: &Node) -> Option<String> {
        match message.kind {
            UnaryMessage { symbol } | UnaryMessagePattern { symbol } => {
                Some(self.symbol_of(&self.find_child(message, symbol)?)?.0)
            }
            BinaryMessage { operator, .. } | BinaryMessagePattern { operator, .. } => {
                if let Operator(t) = self.find_child(message, operator)?.kind {
                    Some(t.lexeme())
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

    fn symbol_of(&self, node: &Node) -> Option<(String, Node)> {
        match node.kind {
            Symbol(ref t) => Some((t.lexeme(), node.clone())),
            Class { symbol, .. }
            | ReferenceTypeExpression { symbol, .. }
            | ReferenceExpression { symbol, .. }
            | TypeParameter { symbol, .. }
            | ParameterPattern { symbol, .. } => {
                self.find_node(symbol).and_then(|s| self.symbol_of(&s))
            }
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

    fn find_usage(&self, node: &Node, kind: DeclarationKind, types: &Types) -> Option<Arc<Usage>> {
        if node.is_declaration(kind) {
            Some(Arc::new(semantics::Usage {
                declaration: node.clone(),
                references: self.find_references(node, kind),
                import_directives: self.find_import_directives_from_declaration(node),
            }))
        } else if node.is_import_directive() {
            let declaration = &self.find_declaration_from_import(&node)?;
            self.find_usage(declaration, declaration.declaration_kind(), types)
        } else if node.is_symbol() {
            let usage_target = &self.usage_target_from_symbol(node)?;
            self.find_usage(usage_target, usage_target.declaration_kind(), types)
        } else if node.is_reference(kind) {
            let declaration = &self.find_declaration(node, node.declaration_kind())?;
            self.find_usage(declaration, declaration.declaration_kind(), types)
        } else if let Method { .. } = node.kind {
            Some(Arc::new(semantics::Usage {
                declaration: node.clone(),
                references: self.find_method_references(node, types),
                import_directives: vec![],
            }))
        } else if node.is_message_pattern() {
            let signature = self.parent(node)?;
            let method = self.parent(&signature)?;
            self.find_usage(&method, DeclarationKind::None, types)
        } else if node.is_message() {
            let method = self.method_from_message(node, types)?;
            self.find_usage(&method, DeclarationKind::None, types)
        } else {
            None
        }
    }

    fn method_from_message(&self, message: &Node, types: &Types) -> Option<Node> {
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

    fn find_method_references(&self, method: &Node, types: &Types) -> Vec<Node> {
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

    fn find_import_directives_from_declaration(&self, declaration: &Node) -> Vec<Node> {
        let mut imports = vec![];

        if let Some((Some(module_id), syntax::Exported(_, _))) =
            self.parent(declaration).map(|n| (n.parent_id, n.kind))
        {
            if let Some(module) = self.find_node_in(&declaration.span.start.uri, module_id) {
                if let Some(namespace) = self.namespace_of_module(&module) {
                    if let Some((name, _)) = self.symbol_of(declaration) {
                        let qualified_exported_name = format!("{}/{}", namespace, name);

                        imports.extend(self.imports_matching(qualified_exported_name));
                    }
                }
            }
        }

        imports
    }

    fn imports_matching(&self, qualified_name: String) -> Vec<Node> {
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

    fn all_message_sends(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_message_send())
    }

    fn all_messages(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_message())
    }

    fn all_imports(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_import_directive())
    }

    fn usage_target_from_symbol(&self, symbol: &Node) -> Option<Node> {
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

    fn find_child(&self, parent: &Node, child_id: Id) -> Option<Node> {
        self.find_node_in(&parent.span.start.uri, child_id)
    }

    fn find_references(&self, declaration: &Node, kind: DeclarationKind) -> Vec<Node> {
        let mut references = vec![];

        if let Some((name, _)) = self.symbol_of(declaration) {
            // If the declaration is exported
            if let Some((module_id, syntax::Exported(_, _))) =
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

            if declaration.is_class() {
                if let Some(class_parent) = self.parent(declaration) {
                    start_scope_root_search_from = class_parent;
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

    fn find_declaration(&self, reference: &Node, kind: DeclarationKind) -> Option<Node> {
        let (name, _) = self.symbol_of(reference)?;
        self.find_declaration_above(reference, name, kind)
    }

    fn find_declaration_above(
        &self,
        node: &Node,
        name: String,
        kind: DeclarationKind,
    ) -> Option<Node> {
        match self.closest_scope_root_upwards(node) {
            None => None,
            Some(scope_root) => {
                let mut result = None;
                self.traverse(&scope_root, &mut |node| {
                    // We do not traverse down scope roots, since
                    // declarations declared there is not reachable
                    // to the original reference.
                    if node.id != scope_root.id && node.is_scope_root() {
                        // Classes exist outside their own scope, though.
                        if node.is_class() {
                            if let Some((n, _)) = self.symbol_of(node) {
                                if n == name {
                                    result = Some(node.clone());
                                }
                            }
                        }

                        return false;
                    }

                    if node.is_import_directive() {
                        if let syntax::ImportDirective {
                            qualified_symbol,
                            symbol,
                            ..
                        } = node.kind
                        {
                            if let Some(qualified_symbol) = self.find_child(node, qualified_symbol)
                            {
                                if let syntax::QualifiedSymbol { ref symbols, .. } =
                                    qualified_symbol.kind
                                {
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

                    true
                });
                if result.is_some() {
                    return result;
                }
                let parent = self.parent(&scope_root)?;
                self.find_declaration_above(&parent, name, kind)
            }
        }
    }

    fn find_references_through_imports(
        &self,
        module: syntax::Node,
        exported_name: String,
        kind: DeclarationKind,
    ) -> Vec<syntax::Node> {
        let mut references = vec![];
        if let Some(namespace) = self.namespace_of_module(&module) {
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

    fn import_directives_of_module(&self, module: &Node) -> Vec<Node> {
        self.all_downwards(module, &|n| n.is_import_directive())
    }

    fn find_declaration_from_import(&self, import_directive: &Node) -> Option<Node> {
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

    fn modules_in_namespace(&self, namespace: String) -> Vec<Node> {
        let namespace = Some(namespace);
        self.modules()
            .into_iter()
            .filter(|module| self.namespace_of_module(module) == namespace)
            .collect()
    }

    fn namespace_of_module(&self, module: &Node) -> Option<String> {
        if let Module {
            namespace_directive,
            ..
        } = module.kind
        {
            if let NamespaceDirective {
                qualified_symbol, ..
            } = self.find_child(module, namespace_directive)?.kind
            {
                return Some(
                    self.qualified_symbol_to_string(&self.find_child(module, qualified_symbol)?),
                );
            }
        }
        None
    }

    fn qualified_symbol_to_string(&self, qualified_symbol: &Node) -> String {
        self.qualified_symbol_to_strings(qualified_symbol).join("/")
    }

    fn qualified_symbol_to_strings(&self, qualified_symbol: &Node) -> Vec<String> {
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

    fn symbol_to_string(&self, symbol: &Node) -> Option<String> {
        if let Symbol(ref t) = symbol.kind {
            Some(t.lexeme())
        } else {
            None
        }
    }

    fn module_declarations_in(&self, module: &Node) -> Vec<(bool, Node)> {
        if let Module {
            ref module_declarations,
            ..
        } = module.kind
        {
            module_declarations
                .iter()
                .filter_map(|mdi| self.find_child(module, *mdi))
                .filter_map(|module_declaration| {
                    if let Exported(_, d) = module_declaration.kind {
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

    fn super_type_expressions(&self, class: &Node) -> Vec<Node> {
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

    fn all_expressions(&self) -> Vec<Node> {
        self.all_matching(|n| n.is_expression())
    }

    fn all_references(&self, kind: DeclarationKind) -> Vec<Node> {
        self.all_matching(|n| n.is_reference(kind))
    }

    fn all_reference_symbols(&self, kind: DeclarationKind) -> Vec<Node> {
        self.all_references(kind)
            .into_iter()
            .filter_map(|reference| self.symbol_of(&reference))
            .map(|(_, n)| n)
            .collect()
    }

    fn all_matching<F: Fn(&Node) -> bool>(&self, f: F) -> Vec<Node> {
        let mut matching = vec![];

        self.traverse_all(&mut |n| {
            if f(n) {
                matching.push(n.clone());
            }
            true
        });

        matching
    }

    fn child_nodes(&self, node: &Node) -> Vec<Node> {
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
    fn traverse<F: FnMut(&Node) -> bool>(&self, from: &Node, f: &mut F) {
        if !f(from) {
            return;
        }

        for child in self.child_nodes(from) {
            self.traverse(&child, f);
        }
    }

    fn closest_upwards<F: Fn(&Node) -> bool>(&self, node: &Node, f: F) -> Option<Node> {
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

    fn all_downwards<F: Fn(&Node) -> bool>(&self, from: &Node, f: &F) -> Vec<Node> {
        let mut nodes = vec![];

        if f(from) {
            nodes.push(from.clone());
        }

        for child in self.child_nodes(from) {
            nodes.extend(self.all_downwards(&child, f));
        }

        nodes
    }

    fn closest_expression_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_expression())
    }

    fn all_expressions_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_expression())
    }

    fn closest_type_expression_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_type_expression())
    }

    fn all_type_expressions_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_type_expression())
    }

    fn closest_scope_root_upwards(&self, from: &Node) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_scope_root())
    }

    fn all_scope_roots_downwards(&self, from: &Node) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_scope_root())
    }

    fn closest_declaration_upwards(&self, from: &Node, kind: DeclarationKind) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_declaration(kind))
    }

    fn all_declarations_downwards(&self, from: &Node, kind: DeclarationKind) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_declaration(kind))
    }

    fn closest_references_upwards(&self, from: &Node, kind: DeclarationKind) -> Option<Node> {
        self.closest_upwards(from, |n| n.is_reference(kind))
    }

    fn all_references_downwards(&self, from: &Node, kind: DeclarationKind) -> Vec<Node> {
        self.all_downwards(from, &|n| n.is_reference(kind))
    }

    fn declaration_is_exported(&self, declaration: &Node) -> bool {
        if let Some(parent) = self.parent(declaration) {
            if let Exported(_, _) = parent.kind {
                return true;
            }
        }
        false
    }
}

#[derive(Clone)]
pub struct ProgramNavigator {
    modules: Arc<HashMap<URI, Arc<Tree>>>,
}

impl ProgramNavigator {
    pub fn new(modules: Arc<HashMap<URI, Arc<Tree>>>) -> ProgramNavigator {
        ProgramNavigator { modules }
    }
}

impl Navigator for ProgramNavigator {
    fn traverse_all<F: FnMut(&Node) -> bool>(&self, f: &mut F) {
        for module in self.modules.values() {
            if let Some(root) = module.root() {
                self.traverse(root, f);
            }
        }
    }

    fn modules(&self) -> Vec<Node> {
        self.modules
            .values()
            .filter_map(|t| t.root())
            .cloned()
            .collect()
    }

    fn find_node(&self, id: Id) -> Option<Node> {
        for (_, tree) in self.modules.iter() {
            if let Some(n) = tree.get(id) {
                return Some(n);
            }
        }
        None
    }

    fn find_node_in(&self, uri: &URI, id: Id) -> Option<Node> {
        self.modules.get(uri).and_then(|t| t.get(id))
    }
}

#[derive(Clone)]
pub struct ModuleNavigator {
    tree: Arc<Tree>,
}

impl ModuleNavigator {
    pub fn new(tree: Arc<Tree>) -> ModuleNavigator {
        ModuleNavigator { tree }
    }
}

impl Navigator for ModuleNavigator {
    fn traverse_all<F: FnMut(&Node) -> bool>(&self, f: &mut F) {
        if let Some(root) = self.tree.root() {
            self.traverse(root, f);
        }
    }

    fn modules(&self) -> Vec<Node> {
        self.tree.root().into_iter().cloned().collect()
    }

    fn find_node(&self, id: Id) -> Option<Node> {
        self.tree.get(id)
    }
}
