use crate::semantics::*;
use crate::syntax::*;
use crate::*;
use std::cmp::min;

#[derive(Clone)]
pub struct Types {
    navigator: Navigator,
    types_cache: Cache<Id, Type>,
}

impl Types {
    pub fn new(navigator: Navigator) -> Types {
        Types {
            navigator,
            types_cache: Cache::new(),
        }
    }

    pub fn attempt_type_coercion(&self, id: Id, to: &Type) -> Option<Type> {
        match self.types_cache.get(&id) {
            None | Some(Type::UnresolvedInteger(_, _)) | Some(Type::UnresolvedFloat(_, _)) => {
                self.types_cache.set(id, to.clone());
                None
            }
            Some(t) => Some(t.clone()),
        }
    }

    pub fn coerced_type(&self, unresolved: &Type) -> Option<Type> {
        match unresolved {
            Type::UnresolvedFloat(_, id) | Type::UnresolvedInteger(_, id) => {
                match self.types_cache.get(id) {
                    Some(Type::UnresolvedFloat(_, _)) | Some(Type::UnresolvedInteger(_, _)) => None,
                    coerced => coerced,
                }
            }
            _ => None,
        }
    }

    pub fn get_type_of_expression(&self, expression: &Node) -> Type {
        self.types_cache
            .gate(&expression.id, || match expression.kind {
                ReferenceExpression { .. } => self.get_expression_type_of_declaration(
                    &self
                        .navigator
                        .find_declaration(expression, DeclarationKind::Value)?,
                ),

                CascadeExpression { expression: e, .. } => {
                    let expression = self.navigator.find_child(expression, e)?;
                    self.get_type_of_expression(&expression)
                }

                TupleExpression { expression: e, .. } => {
                    let expression = self.navigator.find_child(expression, e)?;
                    self.get_type_of_expression(&expression)
                }

                SelfExpression(_) => Type::Self_(Box::new(
                    self.get_type_of_declaration(
                        &self
                            .navigator
                            .find_declaration(expression, DeclarationKind::Value)?,
                    ),
                )),

                StringExpression(_, _) => {
                    self.get_type_of_declaration(&self.navigator.find_stdlib_class("Loa/String")?)
                }

                CharacterExpression(_, _) => self
                    .get_type_of_declaration(&self.navigator.find_stdlib_class("Loa/Character")?),

                IntegerExpression(ref t, _) => Type::UnresolvedInteger(t.lexeme(), expression.id),
                FloatExpression(ref t, _) => Type::UnresolvedFloat(t.lexeme(), expression.id),

                SymbolExpression(ref t, _) => Type::Symbol(t.lexeme()),

                LetExpression { expression, .. } => {
                    let expression = self.navigator.find_node(expression)?;
                    self.get_type_of_expression(&expression)
                }

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
                            return behaviour
                                .with_applied_message(&message, &self.navigator, &self)
                                .return_type()
                                .with_self(&receiver_type);
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

    pub fn get_expression_type_of_declaration(&self, declaration: &Node) -> Type {
        if let Class { .. } = declaration.kind {
            if self.navigator.has_class_object(declaration) {
                return Type::ClassObject(Box::new(self.get_type_of_declaration(declaration)));
            }
        }
        self.get_type_of_declaration(declaration)
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
                LetBinding {
                    type_expression,
                    expression,
                    ..
                } => {
                    if type_expression == Id::NULL {
                        let expression = self.navigator.find_child(declaration, expression)?;
                        self.get_type_of_expression(&expression)
                    } else {
                        let type_expression =
                            self.navigator.find_child(declaration, type_expression)?;
                        self.get_type_of_type_expression(&type_expression)
                    }
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

                SelfTypeExpression(_) => Type::Self_(Box::new(
                    self.get_type_of_declaration(
                        &self
                            .navigator
                            .find_declaration(type_expression, DeclarationKind::Type)?,
                    ),
                )),

                SymbolTypeExpression(ref t, _) => Type::Symbol(t.lexeme()),

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

    pub fn get_behaviours_from_stdlib_class(&self, qn: &str) -> Vec<Behaviour> {
        self.navigator
            .find_stdlib_class(qn)
            .and_then(|class| self.get_behaviours_from_class(&class, &vec![]))
            .unwrap_or(vec![])
    }

    pub fn get_static_behaviours_from_class(
        &self,
        class: &Node,
        class_object_type: &Type,
        class_type: &Type,
    ) -> Option<Vec<Behaviour>> {
        if let Class { class_body, .. } = class.kind {
            let class_body = self.navigator.find_child(class, class_body)?;
            if let ClassBody { class_members, .. } = class_body.kind {
                return Some(
                    class_members
                        .iter()
                        .filter_map(|m| self.navigator.find_child(class, *m))
                        .filter_map(|member| match member.kind {
                            Initializer {
                                message_pattern, ..
                            } => {
                                let message_pattern =
                                    self.navigator.find_child(&member, message_pattern)?;
                                let message =
                                    self.behaviour_message_from_message_pattern(&message_pattern)?;

                                Some(Behaviour {
                                    message,
                                    id: member.id,
                                    receiver_type: class_object_type.clone(),
                                    return_type: class_type.clone(),
                                })
                            }
                            _ => None,
                        })
                        .collect(),
                );
            }
        }
        None
    }

    pub fn get_behaviours(&self, type_: &Type) -> Vec<Behaviour> {
        match type_ {
            Type::Unknown => vec![],
            Type::ClassObject(box class_type) => {
                if let Type::Class(_, class_id, _) = class_type {
                    self.navigator
                        .find_node(*class_id)
                        .and_then(|class| {
                            self.get_static_behaviours_from_class(&class, type_, &class_type)
                        })
                        .unwrap_or(vec![])
                } else {
                    vec![]
                }
            }
            Type::UnresolvedInteger(_, _) => self.get_behaviours_from_stdlib_class("Loa/Integer"),
            Type::UnresolvedFloat(_, _) => self.get_behaviours_from_stdlib_class("Loa/Float"),
            Type::Symbol(_) => self.get_behaviours_from_stdlib_class("Loa/Symbol"),
            Type::Parameter(_, _, _) => vec![],
            Type::Class(_, class_id, args) => self
                .navigator
                .find_node(*class_id)
                .and_then(|class| self.get_behaviours_from_class(&class, args))
                .unwrap_or(vec![]),
            Type::Self_(of) => self.get_behaviours(of),
            Type::Behaviour(box b) => vec![b.clone()],
        }
    }

    pub fn get_behaviours_from_class(
        &self,
        class: &Node,
        args: &Vec<Type>,
    ) -> Option<Vec<Behaviour>> {
        let receiver_type = self.get_type_of_declaration(class);
        if let Class {
            class_body,
            type_parameter_list,
            ..
        } = class.kind
        {
            let type_parameters =
                if let Some(type_parameter_list) = self.navigator.find_node(type_parameter_list) {
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

            let mut behaviours = HashMap::new();

            if let Some(class_body) = self.navigator.find_node(class_body) {
                if let ClassBody {
                    ref class_members, ..
                } = class_body.kind
                {
                    behaviours.extend(
                        class_members
                            .iter()
                            .filter_map(|m| self.navigator.find_child(&class_body, *m))
                            .flat_map(|member| match member.kind {
                                Method { .. } => self
                                    .get_behaviour_from_method(receiver_type.clone(), member)
                                    .map(|m| vec![m])
                                    .unwrap_or(vec![]),

                                Variable { .. } => self
                                    .get_behaviours_from_variable(receiver_type.clone(), &member)
                                    .unwrap_or(vec![]),

                                _ => vec![],
                            })
                            .map(|b| (b.selector(), b.with_applied_type_arguments(&type_arg_map))),
                    );
                }
            }

            for super_type in self.get_super_types(class) {
                for super_behaviour in self.get_behaviours(&super_type) {
                    let selector = super_behaviour.selector();
                    if !behaviours.contains_key(&selector) {
                        behaviours.insert(selector, super_behaviour);
                    }
                }
            }

            return Some(behaviours.into_iter().map(|(_, b)| b).collect());
        }
        None
    }

    fn get_behaviours_from_variable(
        &self,
        receiver_type: Type,
        variable: &Node,
    ) -> Option<Vec<Behaviour>> {
        let (name, _) = self.navigator.symbol_of(variable)?;
        let type_ = self.get_type_of_variable(variable);

        Some(vec![
            Behaviour {
                receiver_type: receiver_type.clone(),
                message: BehaviourMessage::Unary(name.clone()),
                id: variable.id,
                return_type: type_.clone(),
            },
            Behaviour {
                receiver_type: receiver_type.clone(),
                message: BehaviourMessage::Keyword(vec![(name, type_)]),
                id: variable.id,
                return_type: Type::Self_(Box::new(receiver_type)),
            },
        ])
    }

    fn behaviour_message_from_message_pattern(
        &self,
        message_pattern: &Node,
    ) -> Option<BehaviourMessage> {
        match message_pattern.kind {
            UnaryMessagePattern { symbol } => {
                if let Symbol(t) = self.navigator.find_node(symbol)?.kind {
                    Some(BehaviourMessage::Unary(t.lexeme()))
                } else {
                    None
                }
            }

            BinaryMessagePattern {
                operator,
                parameter_pattern,
            } => {
                let parameter_pattern = self.navigator.find_node(parameter_pattern)?;
                let type_ = self.get_type_of_parameter_pattern(&parameter_pattern);

                if let Operator(t) = self.navigator.find_node(operator)?.kind {
                    Some(BehaviourMessage::Binary(
                        t.into_iter()
                            .map(|t| t.lexeme())
                            .collect::<Vec<_>>()
                            .join(""),
                        type_,
                    ))
                } else {
                    None
                }
            }

            KeywordMessagePattern { ref keyword_pairs } => {
                let mut keywords = vec![];

                for pair in keyword_pairs {
                    let pair = self.navigator.find_node(*pair)?;
                    if let KeywordPair { keyword, value, .. } = pair.kind {
                        let keyword = self.navigator.find_node(keyword)?;
                        let (name, _) = self.navigator.symbol_of(&keyword)?;

                        let parameter_pattern = self.navigator.find_node(value)?;
                        let type_ = self.get_type_of_parameter_pattern(&parameter_pattern);

                        keywords.push((name, type_));
                    }
                }

                Some(BehaviourMessage::Keyword(keywords))
            }

            _ => None,
        }
    }

    pub fn get_behaviour_from_method(
        &self,
        receiver_type: Type,
        method: Node,
    ) -> Option<Behaviour> {
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

                let message = self.behaviour_message_from_message_pattern(&message_pattern)?;

                return Some(Behaviour {
                    receiver_type,
                    id: method.id,
                    message,
                    return_type: resolved_return_type,
                });
            }
        }
        None
    }

    pub fn get_behaviour_from_message_send(&self, message_send: &Node) -> Option<Behaviour> {
        if let MessageSendExpression {
            expression,
            message,
            ..
        } = message_send.kind
        {
            let expression = self.navigator.find_child(message_send, expression)?;
            let message = self.navigator.find_child(message_send, message)?;
            let selector = self.navigator.message_selector(&message)?;

            for behaviour in self.get_behaviours(&self.get_type_of_expression(&expression)) {
                if behaviour.selector() == selector {
                    return Some(behaviour);
                }
            }
        }
        None
    }

    pub fn get_type_of_behaviour(&self, behaviour: &Behaviour) -> Type {
        Type::Behaviour(Box::new(behaviour.clone()))
    }

    pub fn get_type_of_variable(&self, variable: &Node) -> Type {
        if let Variable {
            type_expression,
            expression,
            ..
        } = variable.kind
        {
            if type_expression == Id::NULL {
                let expression = self.navigator.find_child(variable, expression)?;
                self.get_type_of_expression(&expression)
            } else {
                let type_expression = self.navigator.find_child(variable, type_expression)?;
                self.get_type_of_type_expression(&type_expression)
            }
        } else {
            Type::Unknown
        }
    }

    fn insert_distanced_types(
        &self,
        distance: usize,
        type_: Type,
        types: &mut HashMap<Type, usize>,
    ) {
        match &type_ {
            Type::Unknown
            | Type::Parameter(_, _, _)
            | Type::Self_(_)
            | Type::Behaviour(_)
            | Type::UnresolvedInteger(_, _)
            | Type::UnresolvedFloat(_, _)
            | Type::Symbol(_)
            | Type::ClassObject(_) => {}
            Type::Class(_, id, _) => {
                if let Some(class) = self.navigator.find_node(*id) {
                    for super_type in self.get_super_types(&class) {
                        self.insert_distanced_types(distance + 1, super_type, types);
                    }
                }
            }
        }
        if let Some(existing_distance) = types.get(&type_) {
            let min_distance = min(distance, *existing_distance);
            types.insert(type_, min_distance);
        } else {
            types.insert(type_, distance);
        }
    }

    fn common_types_ordered_by_distance(&self, types: &Vec<Type>) -> Vec<Type> {
        let mut summarized: HashMap<Type, (usize, usize)> = HashMap::new();

        for (i, type_) in types.iter().enumerate() {
            let mut distances = HashMap::new();
            self.insert_distanced_types(0, type_.clone(), &mut distances);

            for (type_, distance) in distances {
                if i == 0 {
                    summarized.insert(type_, (distance, 1));
                } else if let Some((existing_distance, count)) = summarized.get(&type_).cloned() {
                    summarized.insert(type_, (distance + existing_distance, count + 1));
                }
            }
        }

        let mut to_sort = summarized
            .into_iter()
            .filter_map(|(type_, (distance, count))| {
                if count == types.len() {
                    Some((type_, distance))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        to_sort.sort_by(|(_, a), (_, b)| a.cmp(b));
        to_sort.into_iter().map(|(t, _)| t).collect()
    }

    pub fn get_nearest_common_ancestor(
        &self,
        navigator: &Navigator,
        candidates: Vec<Type>,
    ) -> Type {
        'attempts: for common in self.common_types_ordered_by_distance(&candidates) {
            for candidate in candidates.iter() {
                if check_assignment(common.clone(), candidate.clone(), navigator, self, false)
                    .is_invalid()
                {
                    continue 'attempts;
                }
            }
            return common;
        }

        Type::Unknown
    }

    pub fn get_super_types(&self, class: &Node) -> Vec<Type> {
        let mut super_types = vec![];
        for super_type in self.navigator.super_type_expressions(&class) {
            super_types.push(self.get_type_of_type_expression(&super_type));
        }
        if super_types.len() == 0 {
            if let Some(object_class) = self.navigator.find_stdlib_class("Loa/Object") {
                if object_class.id != class.id {
                    super_types.push(self.get_type_of_declaration(&object_class));
                }
            }
        }
        super_types
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Type {
    Unknown,
    Class(String, Id, Vec<Type>),
    Parameter(String, Id, Vec<Type>),
    Self_(Box<Type>),
    Behaviour(Box<Behaviour>),
    UnresolvedInteger(String, Id),
    UnresolvedFloat(String, Id),
    Symbol(String),
    ClassObject(Box<Type>),
}

impl Type {
    pub fn with_args(self, args: Vec<Type>) -> Type {
        use Type::*;

        match self {
            Self_(box t) => Self_(Box::new(t.with_args(args))),
            Unknown => Unknown,
            Class(s, i, _) => Class(s, i, args),
            Parameter(s, i, _) => Parameter(s, i, args),
            Behaviour(b) => Behaviour(b),
            UnresolvedInteger(s, id) => UnresolvedInteger(s, id),
            UnresolvedFloat(s, id) => UnresolvedFloat(s, id),
            Symbol(s) => Symbol(s),
            ClassObject(i) => ClassObject(i),
        }
    }

    pub fn with_self(self, self_: &Type) -> Type {
        use Type::*;

        match self {
            Self_(_) => self_.clone(),

            // Non-recursive types
            Unknown
            | UnresolvedInteger(_, _)
            | UnresolvedFloat(_, _)
            | Symbol(_)
            | Class(_, _, _)
            | Parameter(_, _, _)
            | ClassObject(_) => self,

            // Recursive types
            Behaviour(box b) => Behaviour(Box::new(b.with_self(self_))),
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
            Type::Behaviour(box b) => Type::Behaviour(Box::new(b.with_applied_type_arguments(map))),
            t => t,
        }
    }

    pub fn to_markdown(&self, navigator: &Navigator) -> String {
        match self {
            Type::ClassObject(ref inner) => format!("{} **class**", inner),
            Type::Symbol(s) => format!("_{}_", s),
            Type::Self_(_) => format!("**self**"),
            Type::Unknown => format!("?"),
            Type::UnresolvedFloat(s, _) => s.clone(),
            Type::UnresolvedInteger(s, _) => s.clone(),
            Type::Class(ref name, id, ref args) | Type::Parameter(ref name, id, ref args) => {
                let start = if let Some(dec) = navigator
                    .find_node(*id)
                    .and_then(|d| Some(navigator.symbol_of(&d)?.1))
                {
                    dec.span.start.clone()
                } else {
                    return self.to_string();
                };

                if args.is_empty() {
                    format!(
                        "[{}]({}#L{},{})",
                        name, start.uri, start.line, start.character
                    )
                } else {
                    format!(
                        "[{}]({}#L{},{})<{}>",
                        name,
                        start.uri,
                        start.line,
                        start.character,
                        args.iter()
                            .map(|t| t.to_markdown(navigator))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            Type::Behaviour(box b) => b.to_markdown(navigator),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::ClassObject(ref inner) => write!(f, "{} class", inner),
            Type::Self_(_) => write!(f, "self"),
            Type::Unknown => write!(f, "?"),
            Type::UnresolvedFloat(s, _) => write!(f, "{}", s),
            Type::UnresolvedInteger(s, _) => write!(f, "{}", s),
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
            Type::Behaviour(b) => write!(f, "{} → {}", b.message, b.return_type),
            Type::Symbol(s) => write!(f, "{}", s),
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

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct Behaviour {
    pub receiver_type: Type,
    pub id: Id,
    pub message: BehaviourMessage,
    pub return_type: Type,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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

    pub fn with_self(self, self_: &Type) -> Behaviour {
        Behaviour {
            receiver_type: self.receiver_type.with_self(self_),
            id: self.id,
            message: match self.message {
                BehaviourMessage::Unary(s) => BehaviourMessage::Unary(s),
                BehaviourMessage::Binary(o, pt) => BehaviourMessage::Binary(o, pt.with_self(self_)),
                BehaviourMessage::Keyword(kws) => BehaviourMessage::Keyword(
                    kws.into_iter()
                        .map(|(s, t)| (s, t.with_self(self_)))
                        .collect(),
                ),
            },
            return_type: self.return_type.with_self(self_),
        }
    }

    pub fn return_type(&self) -> Type {
        self.return_type.clone()
    }

    pub fn with_applied_message(
        self,
        message: &Node,
        navigator: &Navigator,
        types: &Types,
    ) -> Behaviour {
        let mut type_parameter_assignment_candidates: HashMap<Id, Vec<Type>> = HashMap::new();

        match (&self.message, &message.kind) {
            (BehaviourMessage::Unary(_), UnaryMessage { .. }) => {}
            (BehaviourMessage::Binary(_, ref param_type), BinaryMessage { expression, .. }) => {
                if let Type::Parameter(_, id, _) = param_type {
                    if let Some(expression) = navigator.find_child(message, *expression) {
                        let arg_type = types.get_type_of_expression(&expression);

                        if !type_parameter_assignment_candidates.contains_key(id) {
                            type_parameter_assignment_candidates.insert(*id, vec![]);
                        }

                        type_parameter_assignment_candidates
                            .get_mut(id)
                            .unwrap()
                            .push(arg_type);
                    }
                }
            }
            (
                BehaviourMessage::Keyword(ref params),
                KeywordMessage {
                    ref keyword_pairs, ..
                },
            ) => {
                for ((_, param_type), pair) in params.iter().zip(keyword_pairs.iter()) {
                    if let Type::Parameter(_, id, _) = param_type {
                        if let Some(pair) = navigator.find_child(message, *pair) {
                            if let KeywordPair { value, .. } = pair.kind {
                                if let Some(expression) = navigator.find_child(&pair, value) {
                                    let arg_type = types.get_type_of_expression(&expression);

                                    if !type_parameter_assignment_candidates.contains_key(id) {
                                        type_parameter_assignment_candidates.insert(*id, vec![]);
                                    }

                                    type_parameter_assignment_candidates
                                        .get_mut(id)
                                        .unwrap()
                                        .push(arg_type);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        let map = type_parameter_assignment_candidates
            .into_iter()
            .map(|(param_id, candidates)| {
                (
                    param_id,
                    types.get_nearest_common_ancestor(navigator, candidates),
                )
            })
            .collect();

        self.with_applied_type_arguments(&map)
    }

    pub fn with_applied_type_arguments(self, map: &HashMap<Id, Type>) -> Behaviour {
        Behaviour {
            receiver_type: self.receiver_type.with_applied_type_arguments(map),
            id: self.id,
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

    pub fn to_markdown(&self, navigator: &Navigator) -> String {
        match self.message {
            BehaviourMessage::Unary(ref s) => format!(
                "{} **{}** → {}",
                self.receiver_type.to_markdown(navigator),
                s,
                self.return_type.to_markdown(navigator)
            ),
            BehaviourMessage::Binary(ref s, ref t) => format!(
                "{} **{}** {} → {}",
                self.receiver_type.to_markdown(navigator),
                s,
                t.to_markdown(navigator),
                self.return_type.to_markdown(navigator)
            ),
            BehaviourMessage::Keyword(ref kws) => format!(
                "{} {} → {}",
                self.receiver_type.to_markdown(navigator),
                kws.iter()
                    .map(|(s, t)| format!("**{}:** {}", s, t.to_markdown(navigator)))
                    .collect::<Vec<_>>()
                    .join(" "),
                self.return_type.to_markdown(navigator)
            ),
        }
    }
}

impl fmt::Display for BehaviourMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BehaviourMessage::Unary(ref selector) => {
                write!(f, "{}", selector)?;
            }
            BehaviourMessage::Binary(ref operator, ref operand_type) => {
                write!(f, "{} {}", operator, operand_type)?;
            }
            BehaviourMessage::Keyword(ref kwd) => {
                for (i, (arg, type_)) in kwd.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}: {}", arg, type_)?;
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Behaviour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} → {}",
            self.receiver_type, self.message, self.return_type
        )
    }
}
