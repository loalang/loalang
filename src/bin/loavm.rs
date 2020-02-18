use loa::bytecode::BytecodeEncodingRead;
use loa::vm::VM;
use std::env::args;
use std::fs::File;
use std::io::Result;
mod runtime;
use self::runtime::ServerRuntime;

fn main() -> Result<()> {
    for arg in args().skip(1) {
        let instructions = File::open(arg)?.deserialize()?;

        let mut vm = VM::new();
        if let Some(result) = vm.eval_pop::<ServerRuntime>(instructions) {
            println!("{}", result);
        }
    }
    Ok(())
}
