use loa;
use loa::format::Format;

fn main() -> std::io::Result<()> {
    loa::Diagnosed::extract_flat_map(loa::Source::files("**/*.loa")?, |source| {
        loa::syntax::Parser::new(&source).parse_module()
    })
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
