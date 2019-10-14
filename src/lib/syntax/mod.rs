macro_rules! push_all {
    ($children:expr, $other:expr) => {
        $children.extend($other.iter().map(|t| t as &dyn Node));
    };
}

macro_rules! push {
    ($children:expr, $option:expr) => {
        if let Some(ref o) = $option {
            $children.push(o);
        }
    };
}

mod tokens;
pub use self::tokens::*;

mod lexer;
pub use self::lexer::*;

mod node;
pub use self::node::*;

mod tree;
pub use self::tree::*;

mod parser;
pub use self::parser::*;

mod module_cell;
pub use self::module_cell::*;

mod selection;
pub use self::selection::*;

mod scope;
pub use self::scope::*;

pub mod reference_resolver;
