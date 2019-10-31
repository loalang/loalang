mod repl;
pub use self::repl::*;

pub fn repl() {
    let mut repl = repl::REPL::new();

    repl.start();
}
