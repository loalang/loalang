use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub struct Module {
    pub id: Id,
    pub namespace_directive: Option<NamespaceDirective>,
    pub import_directives: Vec<ImportDirective>,
    pub module_declarations: Vec<ModuleDeclaration>,
}

impl Node for Module {
    fn id(&self) -> Option<Id> {
        Some(self.id)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        let first_node: &dyn Node;
        let last_node: &dyn Node;

        if let Some(ref nd) = self.namespace_directive {
            first_node = nd;
        } else if let Some(id) = self.import_directives.first() {
            first_node = id;
        } else if let Some(md) = self.module_declarations.first() {
            first_node = md;
        } else {
            return None;
        }

        if let Some(md) = self.module_declarations.last() {
            last_node = md;
        } else if let Some(id) = self.import_directives.last() {
            last_node = id;
        } else if let Some(ref nd) = self.namespace_directive {
            last_node = nd;
        } else {
            last_node = first_node;
        }

        Some(Span::over(first_node.span()?, last_node.span()?))
    }

    fn children(&self) -> Vec<&dyn Node> {
        let mut children: Vec<&dyn Node> = vec![];

        push!(children, self.namespace_directive);
        push_all!(children, self.import_directives);
        push_all!(children, self.module_declarations);

        children
    }
}
