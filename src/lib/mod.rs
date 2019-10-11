#![feature(box_patterns, try_trait)]

pub use std::borrow::Cow;
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use std::error::Error;
pub use std::fmt;
pub use std::future::Future;
pub use std::slice::Iter;
pub use std::sync::Arc;

extern crate matches;

use matches::*;

extern crate num_bigint;

extern crate glob;

use glob::glob;

mod source;

pub use self::source::*;

mod id;

pub use self::id::*;

#[macro_use]
mod diagnostics;

pub use self::diagnostics::*;

pub mod syntax;

mod program_cell;

pub use self::program_cell::*;
