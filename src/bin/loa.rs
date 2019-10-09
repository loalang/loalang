use loa;

fn main() -> std::io::Result<()> {
    let mut compiler = loa::Compiler::new();
    let mut sources = vec![];
    for arg in std::env::args().skip(1) {
        sources.extend(loa::Source::files(arg)?);
    }
    if sources.len() == 0 {
        println!("No sources.");
        return Ok(());
    }
    match compiler
        .compile_modules(sources)
        .report(&loa::BasicReporter)
    {
        None => Ok(()),
        Some(()) => {
            println!("{}", &compiler.program as &dyn loa::format::Format);
            Ok(())
        }
    }
}
