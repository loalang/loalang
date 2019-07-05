use loa;
use loa::format::Format;

fn main() -> std::io::Result<()> {
    let source = loa::Source::stdin()?;
    loa::syntax::Parser::new(&source)
        .parse_class()
        .map(|e| {
            loa::semantics::Resolver::new().resolve_class(&e)
        })
        .map(|e| {
            println!("{}", &e as &Format);
        });
    Ok(())
}
