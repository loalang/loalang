use crate::pkg::config::{Lockfile, Pkgfile};
use loa::semantics::Analysis;
use loa::syntax::Node;
use loa::syntax::TokenKind;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub struct Versions {
    pub pkgfile: Pkgfile,
    pub lockfile: Lockfile,
}

impl Versions {
    pub fn for_each<F: FnMut(&str, &str)>(&self, mut f: F) {
        f("Loa", env!("CARGO_PKG_VERSION").as_ref());
        if let (Some(ref name), Some(ref version)) = (&self.pkgfile.name, &self.pkgfile.version) {
            f(name.as_ref(), version.as_ref());
        }
        for (name, reg) in self.lockfile.0.iter() {
            f(name.as_ref(), reg.version.as_ref());
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Docs {
    pub classes: BTreeMap<String, ClassDoc>,
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

    pub fn retain_package(&mut self, name: &str) {
        for key in self.classes.keys().cloned().collect::<Vec<_>>() {
            if !key.starts_with(name) {
                self.classes.remove(&key);
            }
        }
    }

    pub fn apply_versions(&mut self, versions: &Versions) {
        let classes = std::mem::replace(&mut self.classes, BTreeMap::new());
        for (mut s, mut class) in classes {
            apply_versions(&mut s, versions);
            class.apply_versions(versions);
            self.classes.insert(s, class);
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
    pub description: Markup,
    pub super_classes: Vec<String>,
    pub sub_classes: Vec<String>,
    pub behaviours: BTreeMap<String, BehaviourDoc>,
}

impl ClassDoc {
    pub fn extract(analysis: &Analysis, class: &Node) -> Option<ClassDoc> {
        Some(ClassDoc {
            name: QualifiedNameDoc::extract(analysis, class)?,
            description: Markup::extract(analysis, class)?,
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

    pub fn apply_versions(&mut self, versions: &Versions) {
        self.name.apply_versions(versions);
        self.super_classes
            .iter_mut()
            .for_each(|s| apply_versions(s, versions));
        self.sub_classes
            .iter_mut()
            .for_each(|s| apply_versions(s, versions));
        self.behaviours
            .values_mut()
            .for_each(|b| b.apply_versions(versions));
    }
}

fn apply_versions(s: &mut String, versions: &Versions) {
    versions.for_each(|name, version| {
        if s.starts_with(name) {
            s.replace_range(name.len()..name.len(), format!("@{}", version).as_ref())
        }
    })
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

    pub fn apply_versions(&mut self, versions: &Versions) {
        apply_versions(&mut self.namespace, versions);
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BehaviourDoc {
    pub selector: String,
    pub description: Markup,
}

impl BehaviourDoc {
    pub fn extract(analysis: &Analysis, method: &Node) -> Option<BehaviourDoc> {
        Some(BehaviourDoc {
            selector: analysis.navigator.method_selector(method)?,
            description: Markup::extract(analysis, method)?,
        })
    }

    pub fn apply_versions(&mut self, _versions: &Versions) {}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Markup {
    pub blocks: Vec<MarkupBlock>,
}

impl Markup {
    pub fn extract(analysis: &Analysis, node: &Node) -> Option<Markup> {
        let comments = node.insignificant_tokens_before(analysis.navigator.tree_of(node)?);

        let mut markup = String::new();
        for comment in comments {
            if let TokenKind::DocComment(content) = comment.kind {
                markup.push_str(content.trim_start());
                markup.push('\n');
            }
        }

        let parser = super::markup::Parser::new(Some((
            analysis.clone(),
            analysis.navigator.closest_scope_root_upwards(node)?,
        )));
        Some(parser.parse_markup(markup))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "__type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarkupBlock {
    Paragraph { elements: Vec<MarkupElement> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "__type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarkupElement {
    Text {
        value: String,
    },
    Bold {
        value: String,
    },
    Italic {
        value: String,
    },
    Link {
        value: String,
        to: String,
    },
    Reference {
        name: QualifiedNameDoc,
        uri: String,
        location: (usize, usize),
    },
}

#[test]
fn applying_version_to_string() {
    let versions = Versions {
        pkgfile: Pkgfile {
            name: Some("Some/Package".into()),
            version: Some("1.0.0".into()),
            dependencies: None,
        },
        lockfile: Lockfile(loa::HashMap::new()),
    };

    let mut s = String::from("Some/Package/Class");
    apply_versions(&mut s, &versions);

    assert_eq!(s, "Some/Package@1.0.0/Class");
}
