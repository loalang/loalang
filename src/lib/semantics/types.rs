use crate::semantics::{Navigator, ProgramNavigator};
use crate::syntax::*;
use crate::*;

#[derive(Clone)]
pub struct Types {
    types_cache: Arc<Mutex<Cache<Id, Type>>>,
    behaviours_cache: Arc<Mutex<Cache<Id, Option<Vec<Behaviour>>>>>,
    navigator: ProgramNavigator,
}

impl Types {
    pub fn new(navigator: ProgramNavigator) -> Types {
        Types {
            navigator,
            types_cache: Arc::new(Mutex::new(Cache::new())),
            behaviours_cache: Arc::new(Mutex::new(Cache::new())),
        }
    }

    fn type_gate<F: FnOnce() -> Type>(&self, node: &Node, f: F) -> Type {
        {
            if let Ok(mut cache) = self.types_cache.lock() {
                if let Some(type_) = cache.get(&node.id) {
                    return type_.clone();
                }

                cache.set(node.id, Type::Unknown);
            }
        }

        let type_ = f();

        {
            if let Ok(mut cache) = self.types_cache.lock() {
                cache.set(node.id, type_.clone());
            }
        }

        type_
    }

    fn behaviours_gate<F: FnOnce() -> Option<Vec<Behaviour>>>(
        &self,
        node: &Node,
        f: F,
    ) -> Option<Vec<Behaviour>> {
        {
            if let Ok(cache) = self.behaviours_cache.lock() {
                if let Some(behaviours) = cache.get(&node.id) {
                    return behaviours.clone();
                }
            }
        }

        let behaviours = f();

        {
            if let Ok(mut cache) = self.behaviours_cache.lock() {
                cache.set(node.id, behaviours.clone());
            }
        }

        behaviours
    }

    pub fn get_type_of_expression(&self, expression: &Node) -> Type {
        self.type_gate(expression, || match expression.kind {
            ReferenceExpression { .. } => self.get_type_of_declaration(
                &self
                    .navigator
                    .find_declaration(expression, DeclarationKind::Value)?,
            ),

            MessageSendExpression {
                expression,
                message,
            } => {
                let receiver = self.navigator.find_node(expression)?;
                let message = self.navigator.find_node(message)?;

                let selector = self.navigator.message_selector(&message)?;

                let receiver_type = self.get_type_of_expression(&receiver);
                let behaviours = self.get_behaviours(&receiver_type);

                for behaviour in behaviours {
                    if behaviour.selector() == selector {
                        return behaviour.return_type();
                    }
                }
                Type::Unknown
            }
            _ => Type::Unknown,
        })
    }

    pub fn get_type_of_declaration(&self, declaration: &Node) -> Type {
        self.type_gate(declaration, || match declaration.kind {
            ParameterPattern {
                type_expression, ..
            } => self.get_type_of_type_expression(&self.navigator.find_node(type_expression)?),
            Class { .. } => {
                let (name, _) = self.navigator.symbol_of(declaration)?;
                Type::Class(name, declaration.id, vec![])
            }
            TypeParameter { .. } => {
                let (name, _) = self.navigator.symbol_of(declaration)?;
                Type::Parameter(name, declaration.id, vec![])
            }
            _ => Type::Unknown,
        })
    }

    pub fn get_type_of_type_expression(&self, type_expression: &Node) -> Type {
        self.type_gate(type_expression, || match type_expression.kind {
            ReferenceTypeExpression {
                type_argument_list, ..
            } => {
                let args = if let Some(argument_list) = self.navigator.find_node(type_argument_list)
                {
                    if let TypeArgumentList {
                        type_expressions, ..
                    } = argument_list.kind
                    {
                        type_expressions
                            .into_iter()
                            .map(|e| {
                                self.navigator
                                    .find_node(e)
                                    .map(|te| self.get_type_of_type_expression(&te))
                                    .unwrap_or(Type::Unknown)
                            })
                            .collect()
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };

                self.get_type_of_declaration(
                    &self
                        .navigator
                        .find_declaration(type_expression, DeclarationKind::Type)?,
                )
                .with_args(args)
            }
            _ => Type::Unknown,
        })
    }

    pub fn get_type_of_method_body(&self, method_body: &Node) -> Type {
        self.type_gate(method_body, || match method_body.kind {
            MethodBody { expression, .. } => {
                self.get_type_of_expression(&self.navigator.find_node(expression)?)
            }
            _ => Type::Unknown,
        })
    }

    pub fn get_type_of_return_type(&self, return_type: &Node) -> Type {
        self.type_gate(return_type, || match return_type.kind {
            ReturnType {
                type_expression, ..
            } => self.get_type_of_type_expression(&self.navigator.find_node(type_expression)?),
            _ => Type::Unknown,
        })
    }

    pub fn get_type_of_parameter_pattern(&self, parameter_pattern: &Node) -> Type {
        self.type_gate(parameter_pattern, || match parameter_pattern.kind {
            ParameterPattern {
                type_expression, ..
            } => self.get_type_of_type_expression(&self.navigator.find_node(type_expression)?),
            _ => Type::Unknown,
        })
    }

    pub fn get_behaviours(&self, type_: &Type) -> Vec<Behaviour> {
        match type_ {
            Type::Unknown => vec![],
            Type::Parameter(_, _, _) => vec![],
            Type::Class(_, class_id, args) => self
                .get_behaviours_from_class(*class_id, args)
                .unwrap_or(vec![]),
        }
    }

    fn get_behaviours_from_class(&self, class_id: Id, args: &Vec<Type>) -> Option<Vec<Behaviour>> {
        let class = self.navigator.find_node(class_id)?;
        self.behaviours_gate(&class, || {
            if let Class {
                class_body,
                type_parameter_list,
                ..
            } = class.kind
            {
                let type_parameters = if let Some(type_parameter_list) =
                    self.navigator.find_node(type_parameter_list)
                {
                    if let TypeParameterList {
                        type_parameters, ..
                    } = type_parameter_list.kind
                    {
                        type_parameters
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                };

                let mut type_arg_map = HashMap::new();
                for (i, param_id) in type_parameters.iter().enumerate() {
                    if i < args.len() {
                        type_arg_map.insert(*param_id, args[i].clone());
                    } else {
                        type_arg_map.insert(*param_id, Type::Unknown);
                    }
                }

                if let ClassBody { class_members, .. } = self.navigator.find_node(class_body)?.kind
                {
                    return Some(
                        class_members
                            .into_iter()
                            .filter_map(|member_id| {
                                let maybe_method = self.navigator.find_node(member_id)?;
                                if let Method { .. } = maybe_method.kind {
                                    self.get_behaviour_from_method(maybe_method)
                                } else {
                                    None
                                }
                            })
                            .map(|b| b.with_applied_type_arguments(&type_arg_map))
                            .collect(),
                    );
                }
            }
            None
        })
    }

    fn get_behaviour_from_method(&self, method: Node) -> Option<Behaviour> {
        if let Method {
            signature,
            method_body,
            ..
        } = method.kind
        {
            let signature = self.navigator.find_node(signature)?;

            if let Signature {
                message_pattern,
                return_type,
            } = signature.kind
            {
                let message_pattern = self.navigator.find_node(message_pattern)?;

                let resolved_return_type = if return_type == Id::NULL {
                    let method_body = self.navigator.find_node(method_body)?;
                    self.get_type_of_method_body(&method_body)
                } else {
                    let return_type = self.navigator.find_node(return_type)?;
                    self.get_type_of_return_type(&return_type)
                };

                match message_pattern.kind {
                    UnaryMessagePattern { symbol } => {
                        if let Symbol(t) = self.navigator.find_node(symbol)?.kind {
                            return Some(Behaviour::Unary(
                                method.id,
                                t.lexeme(),
                                resolved_return_type,
                            ));
                        }
                    }

                    BinaryMessagePattern {
                        operator,
                        parameter_pattern,
                    } => {
                        let parameter_pattern = self.navigator.find_node(parameter_pattern)?;
                        let type_ = self.get_type_of_parameter_pattern(&parameter_pattern);

                        if let Operator(t) = self.navigator.find_node(operator)?.kind {
                            return Some(Behaviour::Binary(
                                method.id,
                                (t.lexeme(), type_),
                                resolved_return_type,
                            ));
                        }
                    }

                    KeywordMessagePattern { keyword_pairs } => {
                        let mut keywords = vec![];

                        for pair in keyword_pairs {
                            let pair = self.navigator.find_node(pair)?;
                            if let KeywordPair { keyword, value, .. } = pair.kind {
                                let keyword = self.navigator.find_node(keyword)?;
                                let (name, _) = self.navigator.symbol_of(&keyword)?;

                                let parameter_pattern = self.navigator.find_node(value)?;
                                let type_ = self.get_type_of_parameter_pattern(&parameter_pattern);

                                keywords.push((name, type_));
                            }
                        }

                        return Some(Behaviour::Keyword(
                            method.id,
                            keywords,
                            resolved_return_type,
                        ));
                    }

                    _ => (),
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum Type {
    Unknown,
    Class(String, Id, Vec<Type>),
    Parameter(String, Id, Vec<Type>),
}

impl Type {
    pub fn with_args(self, args: Vec<Type>) -> Type {
        use Type::*;

        match self {
            Unknown => Unknown,
            Class(s, i, _) => Class(s, i, args),
            Parameter(s, i, _) => Parameter(s, i, args),
        }
    }

    pub fn with_applied_type_arguments(self, map: &HashMap<Id, Type>) -> Type {
        match self {
            Type::Parameter(_, id, _) if map.contains_key(&id) => map[&id].clone(),
            other => other,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Unknown => write!(f, "?"),
            Type::Class(ref name, _, ref args) | Type::Parameter(ref name, _, ref args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(
                        f,
                        "{}<{}>",
                        name,
                        args.iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
        }
    }
}

impl std::ops::Try for Type {
    type Ok = Type;
    type Error = std::option::NoneError;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }

    fn from_error(_v: Self::Error) -> Self {
        Type::Unknown
    }

    fn from_ok(v: Self::Ok) -> Self {
        v
    }
}

#[derive(Debug, Clone)]
pub enum Behaviour {
    Unary(Id, String, Type),
    Binary(Id, (String, Type), Type),
    Keyword(Id, Vec<(String, Type)>, Type),
}

impl Behaviour {
    pub fn selector(&self) -> String {
        match self {
            Behaviour::Unary(_, ref s, _) => s.clone(),
            Behaviour::Binary(_, (ref s, _), _) => s.clone(),
            Behaviour::Keyword(_, ref kws, _) => kws
                .iter()
                .map(|(s, _)| format!("{}:", s))
                .collect::<Vec<_>>()
                .join(""),
        }
    }

    pub fn id(&self) -> Id {
        match self {
            Behaviour::Unary(id, _, _) => *id,
            Behaviour::Binary(id, _, _) => *id,
            Behaviour::Keyword(id, _, _) => *id,
        }
    }

    pub fn return_type(&self) -> Type {
        match self {
            Behaviour::Unary(_, _, ref t) => t.clone(),
            Behaviour::Binary(_, _, ref t) => t.clone(),
            Behaviour::Keyword(_, _, ref t) => t.clone(),
        }
    }

    pub fn with_applied_type_arguments(self, map: &HashMap<Id, Type>) -> Behaviour {
        match self {
            Behaviour::Unary(id, s, t) => {
                Behaviour::Unary(id, s, t.with_applied_type_arguments(map))
            }
            Behaviour::Binary(id, (o, pt), t) => Behaviour::Binary(
                id,
                (o, pt.with_applied_type_arguments(map)),
                t.with_applied_type_arguments(map),
            ),
            Behaviour::Keyword(id, kws, t) => Behaviour::Keyword(
                id,
                kws.into_iter()
                    .map(|(s, t)| (s, t.with_applied_type_arguments(map)))
                    .collect(),
                t.with_applied_type_arguments(map),
            ),
        }
    }
}

impl fmt::Display for Behaviour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Behaviour::Unary(_, ref selector, ref return_type) => {
                write!(f, "{} -> {}", selector, return_type)
            }
            Behaviour::Binary(_, (ref operator, ref operand_type), ref return_type) => {
                write!(f, "{} {} -> {}", operator, operand_type, return_type)
            }
            Behaviour::Keyword(_, ref kwd, ref return_type) => {
                let arguments = kwd.iter().map(|(arg, type_)| format!("{}: {}", arg, type_));
                write!(
                    f,
                    "{} -> {}",
                    arguments.collect::<Vec<_>>().join(" "),
                    return_type
                )
            }
        }
    }
}
