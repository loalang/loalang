use crate::semantics::*;
use crate::syntax::*;
use crate::*;

#[derive(Clone)]
pub struct Types {
    navigator: Navigator,
    types_cache: Cache<Id, Type>,
    behaviours_cache: Cache<Id, Option<Vec<Behaviour>>>,
}

impl Types {
    pub fn new(navigator: Navigator) -> Types {
        Types {
            navigator,
            types_cache: Cache::new(),
            behaviours_cache: Cache::new(),
        }
    }

    pub fn get_type_of_expression(&self, expression: &Node) -> Type {
        self.types_cache
            .gate(&expression.id, || match expression.kind {
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

    pub fn get_types_of_type_parameter_list(&self, list: &Node) -> Option<Vec<Type>> {
        if let TypeParameterList {
            ref type_parameters,
            ..
        } = list.kind
        {
            return Some(
                type_parameters
                    .iter()
                    .map(|pid| {
                        let param = self.navigator.find_child(list, *pid)?;

                        self.get_type_of_declaration(&param)
                    })
                    .collect(),
            );
        }
        None
    }

    pub fn get_type_of_declaration(&self, declaration: &Node) -> Type {
        self.types_cache
            .gate(&declaration.id, || match declaration.kind {
                ParameterPattern {
                    type_expression, ..
                } => self.get_type_of_type_expression(&self.navigator.find_node(type_expression)?),
                Class {
                    type_parameter_list,
                    ..
                } => {
                    let (name, _) = self.navigator.symbol_of(declaration)?;
                    Type::Class(
                        name,
                        declaration.id,
                        self.navigator
                            .find_child(declaration, type_parameter_list)
                            .and_then(|list| self.get_types_of_type_parameter_list(&list))
                            .unwrap_or(vec![]),
                    )
                }
                TypeParameter { .. } => {
                    let (name, _) = self.navigator.symbol_of(declaration)?;
                    Type::Parameter(name, declaration.id, vec![])
                }
                _ => Type::Unknown,
            })
    }

    pub fn get_type_of_type_expression(&self, type_expression: &Node) -> Type {
        self.types_cache
            .gate(&type_expression.id, || match type_expression.kind {
                ReferenceTypeExpression {
                    type_argument_list, ..
                } => {
                    let args =
                        if let Some(argument_list) = self.navigator.find_node(type_argument_list) {
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
        self.types_cache
            .gate(&method_body.id, || match method_body.kind {
                MethodBody { expression, .. } => {
                    self.get_type_of_expression(&self.navigator.find_node(expression)?)
                }
                _ => Type::Unknown,
            })
    }

    pub fn get_type_of_return_type(&self, return_type: &Node) -> Type {
        self.types_cache
            .gate(&return_type.id, || match return_type.kind {
                ReturnType {
                    type_expression, ..
                } => self.get_type_of_type_expression(&self.navigator.find_node(type_expression)?),
                _ => Type::Unknown,
            })
    }

    pub fn get_type_of_parameter_pattern(&self, parameter_pattern: &Node) -> Type {
        self.types_cache
            .gate(&parameter_pattern.id, || match parameter_pattern.kind {
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
        let receiver_type = self.get_type_of_declaration(&class);
        self.behaviours_cache.gate(&class.id, || {
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
                    let mut behaviours: HashMap<String, Behaviour> = class_members
                        .into_iter()
                        .filter_map(|member_id| {
                            let maybe_method = self.navigator.find_node(member_id)?;
                            if let Method { .. } = maybe_method.kind {
                                self.get_behaviour_from_method(receiver_type.clone(), maybe_method)
                            } else {
                                None
                            }
                        })
                        .map(|b| (b.selector(), b.with_applied_type_arguments(&type_arg_map)))
                        .collect();

                    for super_type_expression in self.navigator.super_type_expressions(&class) {
                        for super_behaviour in self.get_behaviours(
                            &self.get_type_of_type_expression(&super_type_expression),
                        ) {
                            let selector = super_behaviour.selector();
                            if !behaviours.contains_key(&selector) {
                                behaviours.insert(selector, super_behaviour);
                            }
                        }
                    }

                    return Some(behaviours.into_iter().map(|(_, b)| b).collect());
                }
            }
            None
        })
    }

    fn get_behaviour_from_method(&self, receiver_type: Type, method: Node) -> Option<Behaviour> {
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
                ..
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

                let message = match message_pattern.kind {
                    UnaryMessagePattern { symbol } => {
                        if let Symbol(t) = self.navigator.find_node(symbol)?.kind {
                            BehaviourMessage::Unary(t.lexeme())
                        } else {
                            return None;
                        }
                    }

                    BinaryMessagePattern {
                        operator,
                        parameter_pattern,
                    } => {
                        let parameter_pattern = self.navigator.find_node(parameter_pattern)?;
                        let type_ = self.get_type_of_parameter_pattern(&parameter_pattern);

                        if let Operator(t) = self.navigator.find_node(operator)?.kind {
                            BehaviourMessage::Binary(t.lexeme(), type_)
                        } else {
                            return None;
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

                        BehaviourMessage::Keyword(keywords)
                    }

                    _ => return None,
                };

                return Some(Behaviour {
                    receiver_type,
                    method_id: method.id,
                    message,
                    return_type: resolved_return_type,
                });
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

    pub fn is_unknown(&self) -> bool {
        matches!(self, Type::Unknown)
    }

    pub fn with_applied_type_arguments(self, map: &HashMap<Id, Type>) -> Type {
        match self {
            Type::Parameter(_, id, _) if map.contains_key(&id) => map[&id].clone(),
            Type::Class(s, i, a) => Type::Class(
                s,
                i,
                a.into_iter()
                    .map(|a| a.with_applied_type_arguments(map))
                    .collect(),
            ),
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

impl Default for Type {
    fn default() -> Self {
        Type::Unknown
    }
}

#[derive(Debug, Clone)]
pub struct Behaviour {
    pub receiver_type: Type,
    pub method_id: Id,
    pub message: BehaviourMessage,
    pub return_type: Type,
}

#[derive(Debug, Clone)]
pub enum BehaviourMessage {
    Unary(String),
    Binary(String, Type),
    Keyword(Vec<(String, Type)>),
}

impl Behaviour {
    pub fn selector(&self) -> String {
        match self.message {
            BehaviourMessage::Unary(ref s) => s.clone(),
            BehaviourMessage::Binary(ref s, _) => s.clone(),
            BehaviourMessage::Keyword(ref kws) => kws
                .iter()
                .map(|(s, _)| format!("{}:", s))
                .collect::<Vec<_>>()
                .join(""),
        }
    }

    pub fn return_type(&self) -> Type {
        self.return_type.clone()
    }

    pub fn with_applied_type_arguments(self, map: &HashMap<Id, Type>) -> Behaviour {
        Behaviour {
            receiver_type: self.receiver_type.with_applied_type_arguments(map),
            method_id: self.method_id,
            message: match self.message {
                BehaviourMessage::Unary(s) => BehaviourMessage::Unary(s),
                BehaviourMessage::Binary(o, pt) => {
                    BehaviourMessage::Binary(o, pt.with_applied_type_arguments(map))
                }
                BehaviourMessage::Keyword(kws) => BehaviourMessage::Keyword(
                    kws.into_iter()
                        .map(|(s, t)| (s, t.with_applied_type_arguments(map)))
                        .collect(),
                ),
            },
            return_type: self.return_type.with_applied_type_arguments(map),
        }
    }
}

impl fmt::Display for Behaviour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", self.receiver_type)?;
        match self.message {
            BehaviourMessage::Unary(ref selector) => {
                write!(f, "{} ", selector)?;
            }
            BehaviourMessage::Binary(ref operator, ref operand_type) => {
                write!(f, "{} {} ", operator, operand_type)?;
            }
            BehaviourMessage::Keyword(ref kwd) => {
                for (arg, type_) in kwd.iter() {
                    write!(f, "{}: {} ", arg, type_)?;
                }
            }
        }
        write!(f, "-> {}", self.return_type)
    }
}
