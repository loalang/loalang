use loa;

fn main() -> std::io::Result<()> {
    let source = loa::Source::file("Makefile".into())?;
    println!("{}:\n{}", source, source.code);
    Ok(())
}
