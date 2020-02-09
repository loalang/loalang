extern crate atty;
use colored::*;
use loa::vm::{CallStack, Runtime, SourceCodeLocation, StackFrame};

pub struct ServerRuntime;

impl Runtime for ServerRuntime {
    fn print_panic(message: String, call_stack: CallStack) {
        let mut call_stack: Vec<_> = call_stack.into();
        call_stack.reverse();
        if atty::is(atty::Stream::Stdout) {
            eprint!("{} ", " PANIC ".bold().white().on_red());
            eprintln!("{}", message.red());
            for StackFrame {
                receiver,
                method,
                callsite: SourceCodeLocation(uri, line, character),
                ..
            } in call_stack
            {
                eprintln!(
                    "{} {}\n  {}",
                    receiver.class.name.yellow(),
                    method.name.yellow(),
                    format!("({}:{}:{})", uri, line, character).bright_black(),
                );
            }
        } else {
            eprintln!("PANIC: {}", message);
            for StackFrame {
                method,
                callsite: SourceCodeLocation(uri, line, character),
                ..
            } in call_stack
            {
                eprintln!("{}\n  ({}:{}:{})", method.name, uri, line, character);
            }
        }
    }
}
