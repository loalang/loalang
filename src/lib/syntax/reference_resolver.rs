use crate::syntax::*;
use crate::*;

pub fn get_references(module: &Module) -> References {
    let mut references = References::new();
    let mut global_scope = Scope::new();

    register_module(&module, &mut global_scope);
    resolve_module(&module, &global_scope, &mut references);

    references
}

fn register_module(module: &Module, scope: &mut Scope) {
    for md in module.module_declarations.iter() {
        register_module_declaration(md, scope);
    }
}

fn resolve_module(module: &Module, scope: &Scope, references: &mut References) {
    for md in module.module_declarations.iter() {
        resolve_module_declaration(md, scope, references);
    }
}

fn register_module_declaration(module_declaration: &ModuleDeclaration, scope: &mut Scope) {
    match module_declaration {
        ModuleDeclaration::Exported(_, d) => register_declaration(d, scope),
        ModuleDeclaration::NotExported(d) => register_declaration(d, scope),
    }
}

fn resolve_module_declaration(
    module_declaration: &ModuleDeclaration,
    scope: &Scope,
    references: &mut References,
) {
    match module_declaration {
        ModuleDeclaration::Exported(_, d) => resolve_declaration(d, scope, references),
        ModuleDeclaration::NotExported(d) => resolve_declaration(d, scope, references),
    }
}

fn register_declaration(declaration: &Declaration, scope: &mut Scope) {
    match declaration {
        Declaration::Class(c) => register_class(c, scope),
    }
}

fn resolve_declaration(declaration: &Declaration, scope: &Scope, references: &mut References) {
    match declaration {
        Declaration::Class(c) => resolve_class(c, scope, references),
    }
}

fn register_class(class: &Class, scope: &mut Scope) {
    if let Some(ref symbol) = class.symbol {
        scope.declare(symbol);
    }
}

fn resolve_class(class: &Class, scope: &Scope, references: &mut References) {
    let mut class_scope = scope.inner();

    // Register
    if let Some(ref class_body) = class.body {
        register_class_body(class_body, &mut class_scope);
    }

    // Resolve
    if let Some(ref class_body) = class.body {
        resolve_class_body(class_body, &class_scope, references);
    }
}

fn register_class_body(class_body: &ClassBody, scope: &mut Scope) {
    for class_member in class_body.class_members.iter() {
        register_class_member(class_member, scope);
    }
}

fn resolve_class_body(class_body: &ClassBody, scope: &Scope, references: &mut References) {
    for class_member in class_body.class_members.iter() {
        resolve_class_member(class_member, scope, references);
    }
}

fn register_class_member(class_member: &ClassMember, scope: &mut Scope) {
    match class_member {
        ClassMember::Method(ref method) => register_method(method, scope),
    }
}

fn resolve_class_member(class_member: &ClassMember, scope: &Scope, references: &mut References) {
    match class_member {
        ClassMember::Method(ref method) => resolve_method(method, scope, references),
    }
}

#[inline]
fn register_method(_method: &Method, _scope: &mut Scope) {
    // A method registers nothing lexically in the class scope.
}

fn resolve_method(method: &Method, scope: &Scope, references: &mut References) {
    let mut method_scope = scope.inner();

    // Register
    register_signature(&method.signature, &mut method_scope);

    if let Some(ref method_body) = method.body {
        register_method_body(method_body, &mut method_scope);
    }

    // Resolve
    resolve_signature(&method.signature, &method_scope, references);

    if let Some(ref method_body) = method.body {
        resolve_method_body(method_body, &method_scope, references);
    }
}

fn register_signature(signature: &Signature, scope: &mut Scope) {
    if let Some(ref message_pattern) = signature.message_pattern {
        register_message_pattern(message_pattern, scope);
    }

    if let Some(ref return_type) = signature.return_type {
        register_return_type(return_type, scope);
    }
}

fn resolve_signature(signature: &Signature, scope: &Scope, references: &mut References) {
    if let Some(ref message_pattern) = signature.message_pattern {
        resolve_message_pattern(message_pattern, scope, references);
    }

    if let Some(ref return_type) = signature.return_type {
        resolve_return_type(return_type, scope, references);
    }
}

fn register_message_pattern(message_pattern: &MessagePattern, _scope: &mut Scope) {
    match message_pattern {
        MessagePattern::Unary(_, _) => (),
    }
}

fn resolve_message_pattern(
    message_pattern: &MessagePattern,
    _scope: &Scope,
    _references: &mut References,
) {
    match message_pattern {
        MessagePattern::Unary(_, _) => (),
    }
}

fn register_return_type(return_type: &ReturnType, scope: &mut Scope) {
    if let Some(ref type_expression) = return_type.type_expression {
        register_type_expression(type_expression, scope);
    }
}

fn resolve_return_type(return_type: &ReturnType, scope: &Scope, references: &mut References) {
    if let Some(ref type_expression) = return_type.type_expression {
        resolve_type_expression(type_expression, scope, references);
    }
}

fn register_method_body(_method_body: &MethodBody, _scope: &mut Scope) {}

fn resolve_method_body(_method_body: &MethodBody, _scope: &Scope, _references: &mut References) {}

fn register_type_expression(type_expression: &TypeExpression, _scope: &mut Scope) {
    match type_expression {
        TypeExpression::Reference(_, _) => (),
    }
}

fn resolve_type_expression(
    type_expression: &TypeExpression,
    scope: &Scope,
    references: &mut References,
) {
    match type_expression {
        TypeExpression::Reference(_, ref s) => resolve_reference(s, scope, references),
    }
}

fn resolve_reference(symbol: &Symbol, scope: &Scope, references: &mut References) {
    if let Some(d) = scope.refer(symbol) {
        references.register_reference(symbol.id, d);
    }
}
