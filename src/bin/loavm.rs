use loa::generation::Instructions;
use loa::vm::VM;
use std::env::args;
use std::fs::read;
use std::io::Result;
mod runtime;
use self::runtime::ServerRuntime;

fn main() -> Result<()> {
    for arg in args().skip(1) {
        let instructions =
            read(arg).map(|bytes| Instructions::from_bytes(bytes.as_slice()).unwrap())?;

        let mut vm = VM::new();
        if let Some(result) = vm.eval_pop::<ServerRuntime>(instructions) {
            println!("{}", result);
        }
    }
    Ok(())
}
