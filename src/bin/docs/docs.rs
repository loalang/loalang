use crate::pkg::config::{Lockfile, Pkgfile};
use loa::semantics::{Analysis, Behaviour, BehaviourMessage, Type};
use loa::syntax::{Node, NodeKind, Token};
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
#[serde(rename_all = "camelCase")]
pub struct ClassDoc {
    pub name: QualifiedNameDoc,
    pub description: Markup,
    pub super_types: Vec<TypeDoc>,
    pub sub_classes: Vec<String>,
    pub behaviours: BTreeMap<String, BehaviourDoc>,
}

impl ClassDoc {
    pub fn extract(analysis: &Analysis, class: &Node) -> Option<ClassDoc> {
        Some(ClassDoc {
            name: QualifiedNameDoc::extract(analysis, class)?,
            description: Markup::extract(analysis, class)?,
            super_types: analysis
                .types
                .get_super_types(class)
                .into_iter()
                .filter_map(|t| TypeDoc::extract(analysis, &t))
                .collect(),
            sub_classes: analysis
                .navigator
                .all_sub_classes_of(class)
                .into_iter()
                .filter_map(|c| Some(analysis.navigator.qualified_name_of(&c)?.0))
                .collect(),
            behaviours: analysis
                .types
                .get_behaviours(&analysis.types.get_type_of_declaration(class))
                .into_iter()
                .filter_map(|behaviour| BehaviourDoc::extract(analysis, &behaviour))
                .map(|b| (b.selector.clone(), b))
                .collect(),
        })
    }

    pub fn apply_versions(&mut self, versions: &Versions) {
        self.name.apply_versions(versions);
        self.sub_classes
            .iter_mut()
            .for_each(|s| apply_versions(s, versions));
        self.super_types
            .iter_mut()
            .for_each(|s| s.apply_versions(versions));
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct QualifiedNameDoc {
    pub name: String,
    pub namespace: String,
}

impl QualifiedNameDoc {
    pub fn extract(analysis: &Analysis, node: &Node) -> Option<QualifiedNameDoc> {
        let (_, namespace, name) = analysis.navigator.qualified_name_of(node)?;
        let namespace = namespace
            .map(|namespace| analysis.navigator.qualified_symbol_to_string(&namespace))
            .unwrap_or(String::new());
        let name = analysis.navigator.symbol_to_string(&name)?;

        Some(QualifiedNameDoc { name, namespace })
    }

    pub fn apply_versions(&mut self, versions: &Versions) {
        apply_versions(&mut self.namespace, versions);
    }
}

impl std::fmt::Display for QualifiedNameDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.namespace.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(f, "{}/{}", self.namespace, self.name)
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BehaviourDoc {
    pub selector: String,
    pub description: Markup,
    pub signature: SignatureDoc,
}

impl BehaviourDoc {
    pub fn extract(analysis: &Analysis, behaviour: &Behaviour) -> Option<BehaviourDoc> {
        Some(BehaviourDoc {
            selector: behaviour.selector(),
            description: Markup::extract(
                analysis,
                &analysis.navigator.find_node(behaviour.method_id)?,
            )?,
            signature: SignatureDoc::extract(analysis, behaviour)?,
        })
    }

    pub fn apply_versions(&mut self, versions: &Versions) {
        self.signature.apply_versions(versions);
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "__type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SignatureDoc {
    #[serde(rename_all = "camelCase")]
    Unary {
        symbol: String,
        return_type: TypeDoc,
    },
    #[serde(rename_all = "camelCase")]
    Binary {
        operator: String,
        operand_type: TypeDoc,
        return_type: TypeDoc,
    },
    #[serde(rename_all = "camelCase")]
    Keyword {
        parameters: Vec<KeywordTypeDoc>,
        return_type: TypeDoc,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KeywordTypeDoc {
    pub keyword: String,
    #[serde(rename = "type")]
    pub type_: TypeDoc,
}

impl SignatureDoc {
    pub fn extract(analysis: &Analysis, behaviour: &Behaviour) -> Option<SignatureDoc> {
        match behaviour.message {
            BehaviourMessage::Unary(ref symbol) => Some(SignatureDoc::Unary {
                symbol: symbol.clone(),
                return_type: TypeDoc::extract(analysis, &behaviour.return_type)?,
            }),
            BehaviourMessage::Binary(ref operator, ref operand) => Some(SignatureDoc::Binary {
                operator: operator.clone(),
                operand_type: TypeDoc::extract(analysis, operand)?,
                return_type: TypeDoc::extract(analysis, &behaviour.return_type)?,
            }),
            BehaviourMessage::Keyword(ref kws) => Some(SignatureDoc::Keyword {
                parameters: kws
                    .iter()
                    .filter_map(|(keyword, type_)| {
                        Some(KeywordTypeDoc {
                            keyword: keyword.clone(),
                            type_: TypeDoc::extract(analysis, type_)?,
                        })
                    })
                    .collect(),
                return_type: TypeDoc::extract(analysis, &behaviour.return_type)?,
            }),
        }
    }

    pub fn apply_versions(&mut self, versions: &Versions) {
        match self {
            SignatureDoc::Unary { return_type, .. } => {
                return_type.apply_versions(versions);
            }
            SignatureDoc::Binary {
                operand_type,
                return_type,
                ..
            } => {
                operand_type.apply_versions(versions);
                return_type.apply_versions(versions);
            }
            SignatureDoc::Keyword {
                parameters,
                return_type,
            } => {
                for param in parameters.iter_mut() {
                    param.type_.apply_versions(versions);
                }
                return_type.apply_versions(versions);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "__type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeDoc {
    Reference {
        class: String,
        arguments: Vec<TypeDoc>,
    },
}

impl TypeDoc {
    pub fn extract(analysis: &Analysis, type_: &Type) -> Option<TypeDoc> {
        match type_ {
            Type::Class(_, class, arguments) => Some(TypeDoc::Reference {
                class: analysis
                    .navigator
                    .qualified_name_of(&analysis.navigator.find_node(*class)?)?
                    .0,
                arguments: arguments
                    .iter()
                    .filter_map(|a| TypeDoc::extract(analysis, a))
                    .collect(),
            }),
            _ => None,
        }
    }

    pub fn apply_versions(&mut self, versions: &Versions) {
        match self {
            TypeDoc::Reference { class, arguments } => {
                apply_versions(class, versions);
                for arg in arguments.iter_mut() {
                    arg.apply_versions(versions);
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Markup {
    pub blocks: Vec<MarkupBlock>,
}

impl Markup {
    pub fn extract(analysis: &Analysis, node: &Node) -> Option<Markup> {
        let mut blocks = vec![];
        if let Some(doc) = analysis.navigator.doc_of(node) {
            for block in analysis.navigator.blocks_of_doc(&doc) {
                blocks.push(MarkupBlock::extract(analysis, &block)?);
            }
        }
        Some(Markup { blocks })
    }

    pub fn to_markdown(&self) -> String {
        let mut output = String::new();

        for (i, block) in self.blocks.iter().enumerate() {
            if i > 0 {
                output.push_str("\n\n");
            }

            output.push_str(block.to_markdown().as_str());
        }

        output
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "__type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarkupBlock {
    Paragraph { elements: Vec<MarkupElement> },
}

impl MarkupBlock {
    pub fn extract(analysis: &Analysis, block: &Node) -> Option<MarkupBlock> {
        match block.kind {
            NodeKind::DocParagraphBlock { ref elements } => Some(MarkupBlock::Paragraph {
                elements: elements
                    .into_iter()
                    .filter_map(|i| analysis.navigator.find_child(block, *i))
                    .filter_map(|n| MarkupElement::extract(analysis, &n))
                    .collect(),
            }),
            _ => None,
        }
    }

    pub fn to_markdown(&self) -> String {
        match self {
            MarkupBlock::Paragraph { elements } => {
                let mut output = String::new();
                for element in elements.iter() {
                    output.push_str(element.to_markdown().as_str());
                }
                output
            }
        }
    }
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

impl MarkupElement {
    pub fn extract(analysis: &Analysis, element: &Node) -> Option<MarkupElement> {
        match element.kind {
            NodeKind::DocTextElement(ref tokens) => Some(MarkupElement::Text {
                value: tokens.iter().map(Token::lexeme).collect(),
            }),
            NodeKind::DocItalicElement(_, ref tokens, _) => Some(MarkupElement::Italic {
                value: tokens.iter().map(Token::lexeme).collect(),
            }),
            NodeKind::DocBoldElement(_, ref tokens, _) => Some(MarkupElement::Bold {
                value: tokens.iter().map(Token::lexeme).collect(),
            }),
            NodeKind::DocLinkElement(text, re) => {
                let text = analysis.navigator.find_child(element, text)?;

                if let NodeKind::DocLinkText(_, ref tokens, _) = text.kind {
                    let value: String = tokens.iter().map(Token::lexeme).collect();
                    let to;
                    if let Some(NodeKind::DocLinkRef(_, ref tokens, _)) =
                        analysis.navigator.find_child(element, re).map(|n| n.kind)
                    {
                        to = tokens.iter().map(Token::lexeme).collect();
                    } else {
                        to = value.clone();
                    }

                    Some(MarkupElement::Link { value, to })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn to_markdown(&self) -> String {
        match self {
            MarkupElement::Text { value } => value.clone(),
            MarkupElement::Bold { value } => format!("**{}**", value),
            MarkupElement::Italic { value } => format!("_{}_", value),
            MarkupElement::Link { to, value } => format!("[{}]({})", value, to),
            MarkupElement::Reference {
                name,
                uri,
                location: (l, c),
            } => format!("[{}]({}#L{},{})", name, uri, l, c),
        }
    }
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
