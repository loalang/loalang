#![feature(box_patterns, try_trait, matches_macro)]

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

extern crate glob;

extern crate fraction;
extern crate num_bigint;
extern crate num_traits;
pub use fraction::BigFraction;
pub use num_bigint::{BigInt, BigUint};
pub use num_traits::pow::Pow;

extern crate bincode;

extern crate peekmore;

extern crate strum;
#[macro_use]
extern crate strum_macros;

#[macro_use]
extern crate matches;

mod source;

pub use self::source::*;

mod id;
pub use self::id::*;

mod cache;
pub use self::cache::*;

mod stack;
pub use self::stack::*;

mod bit_size;
pub use self::bit_size::*;

#[macro_use]
mod diagnostics;

pub use self::diagnostics::*;

pub mod syntax;

pub mod semantics;

pub mod generation;

pub mod vm;

pub mod server;

pub mod format;

pub mod assembly;

pub mod bytecode;

#[cfg(test)]
mod fixture_tests;
