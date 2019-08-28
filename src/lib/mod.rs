#![feature(box_patterns)]

pub use std::borrow::Cow;
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use std::fmt;
pub use std::slice::Iter;
pub use std::sync::Arc;

extern crate matches;
use matches::*;

extern crate num_bigint;
use num_bigint::BigInt;

extern crate glob;
use glob::glob;

mod source;
pub use self::source::*;

#[macro_use]
mod diagnostics;
pub use self::diagnostics::*;

pub mod syntax;

pub mod semantics;

pub mod format;
