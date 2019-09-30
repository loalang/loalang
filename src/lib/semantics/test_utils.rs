use crate::semantics::*;
use crate::*;

pub fn symbol(name: &str) -> Symbol {
    Symbol(None, name.into())
}

pub fn class<F: FnOnce(&mut Class)>(name: &str, f: F) -> Arc<Class> {
    let mut class = Class {
        name: Symbol(None, name.into()),
        qualified_name: name.into(),
        type_parameters: vec![],
        super_types: vec![],
        variables: vec![],
        methods: vec![],
    };
    f(&mut class);
    Arc::new(class)
}

pub fn class_type(class: Arc<Class>) -> Type {
    Type {
        constructor: TypeConstructor::Class(Arc::into_raw(class)),
        arguments: vec![],
    }
}

pub fn type_parameter_type(type_parameter: Arc<TypeParameter>) -> Type {
    Type {
        constructor: TypeConstructor::TypeParameter(Arc::into_raw(type_parameter)),
        arguments: vec![],
    }
}

pub fn proper_type(name: &str) -> Type {
    class_type(class(name, |_| {}))
}

pub fn partial_unary_method(selector: &str, return_type: Type) -> Method {
    Method {
        visibility: Visibility::Public,
        signature: Signature {
            selector: Symbol(None, selector.into()),
            type_parameters: vec![],
            parameters: vec![],
            return_type,
        },
        implementation: None,
    }
}

pub fn type_parameter<F: FnOnce(&mut TypeParameter)>(
    constraint: Type,
    name: &str,
    f: F,
) -> Arc<TypeParameter> {
    let mut param = TypeParameter {
        constraint,
        name: Symbol(None, name.into()),
        type_parameters: vec![],
        variance: Variance::Invariant,
    };
    f(&mut param);
    Arc::new(param)
}

pub fn format_methods(methods: HashMap<Symbol, Method>) -> Vec<String> {
    let mut methods = methods
        .iter()
        .map(|(_, m)| format!("{}", m.signature))
        .collect::<Vec<_>>();

    methods.sort();

    methods
}
