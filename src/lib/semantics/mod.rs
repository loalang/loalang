mod analysis;
pub use self::analysis::*;

mod usage;
pub use self::usage::*;

mod navigator;
pub use self::navigator::*;

mod types;
pub use self::types::*;

mod type_assignability;
pub use self::type_assignability::*;

mod checker;
pub use self::checker::*;

pub mod checkers;

use crate::*;
use std::time::{Duration, Instant};

const CACHE_CANDIDATE_WARNING_LIMIT: Duration = Duration::from_millis(10);

fn cache_candidate<T, F: FnOnce() -> T>(name: &str, f: F) -> T {
    let now = Instant::now();
    let result = f();
    if now.elapsed() > CACHE_CANDIDATE_WARNING_LIMIT {
        warn!("Cache candidate {:?} took {:?}.", name, now.elapsed());
    }
    result
}
