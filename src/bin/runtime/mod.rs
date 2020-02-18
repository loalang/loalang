extern crate atty;
use colored::*;
use loa::vm::{CallStack, Runtime, SourceCodeLocation, StackFrame};

pub struct ServerRuntime;

impl Runtime for ServerRuntime {
    fn print_panic(message: String, call_stack: CallStack) {
        let call_stack: Vec<_> = call_stack.into();
        if atty::is(atty::Stream::Stdout) {
            eprint!("{} ", " PANIC ".bold().white().on_red());
            eprintln!("{}", message.red());
            for frame in call_stack {
                let StackFrame {
                    receiver,
                    method,
                    callsite: SourceCodeLocation(uri, line, character),
                    ..
                } = frame.as_ref();
                eprintln!(
                    "{} {}\n  {}",
                    receiver
                        .class
                        .as_ref()
                        .map(|c| c.name.as_ref())
                        .unwrap_or("?")
                        .yellow(),
                    method.name.yellow(),
                    format!("({}:{}:{})", uri, line, character).bright_black(),
                );
            }
        } else {
            eprintln!("PANIC: {}", message);
            for frame in call_stack {
                let StackFrame {
                    receiver,
                    method,
                    callsite: SourceCodeLocation(uri, line, character),
                    ..
                } = frame.as_ref();
                eprintln!(
                    "{} {}\n  ({}:{}:{})",
                    receiver
                        .class
                        .as_ref()
                        .map(|c| c.name.as_ref())
                        .unwrap_or("?"),
                    method.name,
                    uri,
                    line,
                    character
                );
            }
        }
    }
}
