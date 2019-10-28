use crate::syntax::Node;
use crate::*;

#[derive(Clone)]
pub struct Types {
    cache: Arc<Mutex<Cache<Id, Type>>>,
}

impl Types {
    pub fn new() -> Types {
        Types {
            cache: Arc::new(Mutex::new(Cache::new())),
        }
    }

    fn gate<F: FnOnce() -> Type>(&self, node: &Node, f: F) -> Type {
        {
            if let Ok(cache) = self.cache.lock() {
                if let Some(type_) = cache.get(&node.id) {
                    return type_.clone();
                }
            }
        }

        let type_ = f();

        {
            if let Ok(mut cache) = self.cache.lock() {
                cache.set(node.id, type_.clone());
            }
        }

        type_
    }

    pub fn get_type_of_expression(&self, expression: &Node) -> Type {
        self.gate(expression, || Type::Unknown)
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Unknown,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Unknown => write!(f, "?"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Behaviour {
    Unary(String, Type),
    Binary((String, Type), Type),
    Keyword(Vec<(String, Type)>, Type),
}

impl Behaviour {
    pub fn selector(&self) -> String {
        match self {
            Behaviour::Unary(ref s, _) => s.clone(),
            Behaviour::Binary((ref s, _), _) => s.clone(),
            Behaviour::Keyword(ref kws, _) => kws
                .iter()
                .map(|(s, _)| format!("{}:", s))
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}

impl fmt::Display for Behaviour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Behaviour::Unary(ref selector, ref return_type) => {
                write!(f, "{} -> {}", selector, return_type)
            }
            Behaviour::Binary((ref operator, ref operand_type), ref return_type) => {
                write!(f, "{} {} -> {}", operator, operand_type, return_type)
            }
            Behaviour::Keyword(ref kwd, ref return_type) => {
                let arguments = kwd.iter().map(|(arg, type_)| format!("{}: {}", arg, type_));
                write!(
                    f,
                    "{} -> {}",
                    arguments.collect::<Vec<_>>().join(" "),
                    return_type
                )
            }
        }
    }
}
