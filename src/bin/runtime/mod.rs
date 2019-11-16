extern crate atty;
use colored::*;
use loa::vm::{CallStack, Runtime};

pub struct ServerRuntime;

impl Runtime for ServerRuntime {
    fn print_panic(message: String, mut call_stack: CallStack) {
        call_stack.reverse();
        if atty::is(atty::Stream::Stdout) {
            eprint!("{} ", " PANIC ".bold().white().on_red());
            eprintln!("{}", message.red());
            for (location, class, method) in call_stack {
                eprintln!(
                    "{} {}\n  {}{}{}",
                    class.name.bright_red(),
                    method.name.yellow(),
                    "(".bright_black(),
                    location.bright_black(),
                    ")".bright_black(),
                );
            }
        } else {
            eprintln!("PANIC: {}", message);
            for (location, class, method) in call_stack {
                eprintln!("{} {}\n  ({})", class.name, method.name, location);
            }
        }
    }
}
