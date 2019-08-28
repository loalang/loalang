use loa;
use loa::format::Format;

fn main() -> std::io::Result<()> {
    let source = loa::Source::stdin()?;
    loa::syntax::Parser::new(&source)
        .parse_module()
        .map(|m| loa::semantics::Resolver::new().resolve_modules(&vec![m]))
        .flat_map(|p| {
            let mut global_scope = loa::semantics::LexicalScope::new();
            global_scope.register_program(&p);
            global_scope.resolve_program(p)
        })
        .map(|p| {
            println!("{}", &p as &dyn Format);
        })
        .report(&loa::BasicReporter);
    Ok(())
}
