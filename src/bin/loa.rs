use loa;
use loa::format::Format;

fn main() -> std::io::Result<()> {
    loa::syntax::Parser::parse_modules(loa::Source::files("**/*.loa")?)
        .map(|m| loa::semantics::Resolver::new().resolve_modules(&m))
        .flat_map(|p| {
            let mut global_scope = loa::semantics::LexicalScope::new();
            global_scope.register_program(&p);
            global_scope.resolve_program(p)
        })
        .flat_map(|p| {
            let mut resolver = loa::semantics::TypeResolver::new();
            resolver.resolve_program(&p);
            loa::Diagnosed::Diagnosis(p, resolver.diagnostics)
        })
        .map(|p| {
            println!("{}", &p as &dyn Format);
            p
        })
        .report(&loa::BasicReporter);
    Ok(())
}
