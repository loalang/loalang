use loa::semantics::Analysis;
use loa::syntax::Node;
use loa::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Docs {
    pub classes: HashMap<String, ClassDoc>,
}

impl Docs {
    pub fn extract(analysis: &Analysis) -> Docs {
        Docs {
            classes: analysis
                .navigator
                .all_classes()
                .into_iter()
                .filter_map(|class| ClassDoc::extract(analysis, &class))
                .map(|d| (format!("{}/{}", d.name.namespace, d.name.name), d))
                .collect(),
        }
    }
}

impl From<Analysis> for Docs {
    fn from(analysis: Analysis) -> Self {
        Docs::extract(&analysis)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClassDoc {
    pub name: QualifiedNameDoc,
    pub super_classes: Vec<String>,
    pub sub_classes: Vec<String>,
    pub behaviours: HashMap<String, BehaviourDoc>,
}

impl ClassDoc {
    pub fn extract(analysis: &Analysis, class: &Node) -> Option<ClassDoc> {
        Some(ClassDoc {
            name: QualifiedNameDoc::extract(analysis, class)?,
            super_classes: analysis
                .navigator
                .all_super_classes_of(class)
                .into_iter()
                .filter_map(|c| Some(analysis.navigator.qualified_name_of(&c)?.0))
                .collect(),
            sub_classes: analysis
                .navigator
                .all_sub_classes_of(class)
                .into_iter()
                .filter_map(|c| Some(analysis.navigator.qualified_name_of(&c)?.0))
                .collect(),
            behaviours: analysis
                .navigator
                .methods_of_class(class)
                .into_iter()
                .filter_map(|method| BehaviourDoc::extract(analysis, &method))
                .map(|b| (b.selector.clone(), b))
                .collect(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct QualifiedNameDoc {
    pub name: String,
    pub namespace: String,
}

impl QualifiedNameDoc {
    pub fn extract(analysis: &Analysis, node: &Node) -> Option<QualifiedNameDoc> {
        let (_, namespace, name) = analysis.navigator.qualified_name_of(node)?;
        let namespace = namespace?;
        let namespace = analysis.navigator.qualified_symbol_to_string(&namespace);
        let name = analysis.navigator.symbol_to_string(&name)?;

        Some(QualifiedNameDoc { name, namespace })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BehaviourDoc {
    pub selector: String,
}

impl BehaviourDoc {
    pub fn extract(analysis: &Analysis, method: &Node) -> Option<BehaviourDoc> {
        Some(BehaviourDoc {
            selector: analysis.navigator.method_selector(method)?,
        })
    }
}
