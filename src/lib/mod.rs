#![feature(box_patterns, try_trait)]

pub use std::any::Any;
pub use std::borrow::Cow;
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use std::error::Error;
pub use std::fmt;
pub use std::future::Future;
pub use std::slice::Iter;
pub use std::sync::Arc;
pub use std::sync::Mutex;

extern crate log;

pub use log::*;

extern crate matches;

use matches::*;

extern crate num_bigint;

extern crate glob;

use glob::glob;

mod source;

pub use self::source::*;

mod id;
pub use self::id::*;

mod cache;
pub use self::cache::*;

#[macro_use]
mod diagnostics;

pub use self::diagnostics::*;

pub mod syntax;

pub mod semantics;

pub mod server;
