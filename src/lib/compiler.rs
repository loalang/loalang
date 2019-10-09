use crate::*;
use crate::semantics::TypeValidator;

pub struct Compiler {
    pub program: semantics::Program,
    global_scope: semantics::LexicalScope<'static>,
    type_resolver: semantics::TypeResolver,
}

impl Compiler {
    pub fn new() -> Compiler {
        Compiler {
            program: semantics::Program { classes: vec![] },
            global_scope: semantics::LexicalScope::new(),
            type_resolver: semantics::TypeResolver::new(),
        }
    }

    pub fn compile_modules(&mut self, sources: Vec<Arc<Source>>) -> Diagnosed<()> {
        let mut diagnostics = vec![];

        for source in sources {
            diagnose!(diagnostics, self.compile_module(source));
        }

        Diagnosed::maybe_diagnosis((), diagnostics)
    }

    pub fn compile_module(&mut self, source: Arc<Source>) -> Diagnosed<()> {
        let mut diagnostics = vec![];

        let mut parser = syntax::Parser::new(&source);
        let module = diagnose!(diagnostics, parser.parse_module());
        let mut resolver = semantics::Resolver::new();
        resolver.resolve_module_into_program(&mut self.program, &module);

        self.global_scope.register_program(&self.program);

        let mut program =
            std::mem::replace(&mut self.program, semantics::Program { classes: vec![] });
        program = diagnose!(diagnostics, self.global_scope.resolve_program(program));
        self.type_resolver.resolve_program(&program);

        self.program = program;

        let mut diagnostics = vec![];
        diagnose!(diagnostics, Diagnosed::flush(&mut self.type_resolver.diagnostics));

        let validator = TypeValidator::from_resolver(&self.type_resolver);
        diagnose!(diagnostics, validator.validate_program(&self.program));
        Diagnosed::maybe_diagnosis((), diagnostics)
    }
}
