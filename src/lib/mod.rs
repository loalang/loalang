#![feature(termination_trait_lib)]
#![feature(box_patterns)]

pub use std::sync::Arc;

extern crate matches;
use matches::*;

mod source;
pub use self::source::*;

#[macro_use]
mod diagnostics;
pub use self::diagnostics::*;

pub mod syntax;
