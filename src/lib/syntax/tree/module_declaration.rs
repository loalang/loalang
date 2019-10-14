use crate::syntax::*;
use crate::*;

#[derive(Debug)]
pub enum ModuleDeclaration {
    Exported(Token, Declaration),
    NotExported(Declaration),
}

impl Node for ModuleDeclaration {
    fn id(&self) -> Option<Id> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn span(&self) -> Option<Span> {
        match self {
            ModuleDeclaration::Exported(_, ref d) => d.span(),
            ModuleDeclaration::NotExported(ref d) => d.span(),
        }
    }

    fn children(&self) -> Vec<&dyn Node> {
        match self {
            ModuleDeclaration::Exported(_, ref d) => vec![d],
            ModuleDeclaration::NotExported(ref d) => vec![d],
        }
    }
}
