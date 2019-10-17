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

fn register_message_pattern(message_pattern: &MessagePattern, scope: &mut Scope) {
    match message_pattern {
        MessagePattern::Unary(_, _) => (),
        MessagePattern::Binary(_, _, p) => register_parameter_pattern(p, scope),
        MessagePattern::Keyword(_, kw) => register_keyworded(kw, register_parameter_pattern, scope),
    }
}

fn resolve_message_pattern(
    message_pattern: &MessagePattern,
    scope: &Scope,
    references: &mut References,
) {
    match message_pattern {
        MessagePattern::Unary(_, _) => (),
        MessagePattern::Binary(_, _, p) => resolve_parameter_pattern(p, scope, references),
        MessagePattern::Keyword(_, kw) => {
            resolve_keyworded(kw, resolve_parameter_pattern, scope, references)
        }
    }
}

fn register_keyworded<T, F: Fn(&T, &mut Scope)>(
    keyworded: &Keyworded<T>,
    register_item: F,
    scope: &mut Scope,
) {
    for (_, _, item) in keyworded.keywords.iter() {
        register_item(item, scope);
    }
}

fn resolve_keyworded<T, F: Fn(&T, &Scope, &mut References)>(
    keyworded: &Keyworded<T>,
    resolve_item: F,
    scope: &Scope,
    references: &mut References,
) {
    for (_, _, item) in keyworded.keywords.iter() {
        resolve_item(item, scope, references);
    }
}

fn register_parameter_pattern(pattern: &ParameterPattern, scope: &mut Scope) {
    match pattern {
        ParameterPattern::Nothing(_, _) => (),
        ParameterPattern::Parameter(_, t, s) => {
            if let Some(t) = t {
                register_type_expression(t, scope);
            }

            if let Some(s) = s {
                scope.declare(s);
            }
        }
    }
}

fn resolve_parameter_pattern(
    pattern: &ParameterPattern,
    scope: &Scope,
    references: &mut References,
) {
    match pattern {
        ParameterPattern::Nothing(_, _) => (),
        ParameterPattern::Parameter(_, t, _) => {
            if let Some(t) = t {
                resolve_type_expression(t, scope, references);
            }
        }
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

fn register_method_body(method_body: &MethodBody, scope: &mut Scope) {
    if let Some(ref expression) = method_body.expression {
        register_expression(expression, scope);
    }
}

fn resolve_method_body(method_body: &MethodBody, scope: &Scope, references: &mut References) {
    if let Some(ref expression) = method_body.expression {
        resolve_expression(expression, scope, references);
    }
}

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

fn register_expression(expression: &Expression, scope: &mut Scope) {
    match expression {
        Expression::Reference(_, _) => (),
        Expression::MessageSend(_, box receiver, box message) => {
            register_expression(receiver, scope);
            register_message(message, scope);
        }
    }
}

fn resolve_expression(expression: &Expression, scope: &Scope, references: &mut References) {
    match expression {
        Expression::Reference(_, ref s) => resolve_reference(s, scope, references),
        Expression::MessageSend(_, box receiver, box message) => {
            resolve_expression(receiver, scope, references);
            resolve_message(message, scope, references);
        }
    }
}

fn register_message(message: &Message, scope: &mut Scope) {
    match message {
        Message::Unary(_, _) => (),
        Message::Binary(_, _, ref e) => register_expression(e, scope),
        Message::Keyword(_, ref kw) => register_keyworded(kw, register_expression, scope),
    }
}

fn resolve_message(message: &Message, scope: &Scope, references: &mut References) {
    match message {
        Message::Unary(_, _) => (),
        Message::Binary(_, _, ref e) => resolve_expression(e, scope, references),
        Message::Keyword(_, ref kw) => resolve_keyworded(kw, resolve_expression, scope, references),
    }
}
