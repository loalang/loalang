#![feature(termination_trait_lib)]
#![feature(box_patterns)]

pub use std::sync::Arc;
pub use std::slice::Iter;
pub use std::collections::HashMap;
pub use std::fmt;

extern crate matches;
use matches::*;

extern crate num_bigint;
use num_bigint::{BigInt};

mod source;
pub use self::source::*;

#[macro_use]
mod diagnostics;
pub use self::diagnostics::*;

pub mod syntax;

pub mod semantics;
