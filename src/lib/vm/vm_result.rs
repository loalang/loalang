use crate::vm::*;

macro_rules! expect {
    ($vm:expr, $opt:expr, $($arg:tt)*) => {
        match $opt {
            Some(t) => t,
            None => return VMResult::Panic(format!($($arg)*), $vm.call_stack.detach()),
        }
    };
}

macro_rules! unwrap {
    ($vm:expr, $opt:expr) => {
        match $opt {
            VMResult::Ok(t) => t,
            VMResult::Panic(s, cs) => return VMResult::Panic(s, cs),
        }
    };
}

pub enum VMResult<T> {
    Ok(T),
    Panic(String, CallStack),
}
