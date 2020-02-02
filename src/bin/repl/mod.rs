mod repl;
pub use self::repl::*;

pub fn repl(use_std: bool) {
    let mut repl = repl::REPL::new::<crate::PrettyReporter>(use_std);

    repl.start::<crate::PrettyReporter>();
}
