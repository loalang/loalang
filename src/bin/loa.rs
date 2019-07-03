use loa;

fn main() -> std::io::Result<()> {
    let source = loa::Source::stdin()?;
    println!("{:?}", loa::syntax::Parser::new(&source).parse_expression());
    Ok(())
}
