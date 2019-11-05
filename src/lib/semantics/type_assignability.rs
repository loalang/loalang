use crate::semantics::*;
use crate::syntax::*;
use crate::*;

#[derive(Clone, Debug)]
pub enum TypeAssignability {
    Valid,
    Invalid {
        assignee: Type,
        assigned: Type,
        invariant: bool,
        because: Vec<TypeAssignability>,
    },
    Uncoercable {
        from: Type,
        to: Type,
        already_coerced_to: Option<Type>,
        because: Vec<TypeAssignability>,
    },
}

impl TypeAssignability {
    pub fn is_valid(&self) -> bool {
        if let TypeAssignability::Valid = self {
            true
        } else {
            false
        }
    }

    pub fn is_invalid(&self) -> bool {
        match self {
            TypeAssignability::Invalid { .. } => true,
            TypeAssignability::Uncoercable { .. } => true,
            _ => false,
        }
    }

    pub fn expect<F: FnOnce(Vec<TypeAssignability>) -> TypeAssignability>(
        self,
        f: F,
    ) -> TypeAssignability {
        if self.is_valid() {
            self
        } else {
            f(vec![self])
        }
    }
}

impl std::ops::Try for TypeAssignability {
    type Ok = TypeAssignability;
    type Error = std::option::NoneError;

    fn into_result(self) -> Result<Self::Ok, Self::Error> {
        Ok(self)
    }

    fn from_error(_v: Self::Error) -> Self {
        // We gracefully make a NoneError into
        // a valid type assignability, because
        // it occurs when navigating the AST
        // failed, which should be addressed
        // by other diagnostics.
        TypeAssignability::Valid
    }

    fn from_ok(v: Self::Ok) -> Self {
        v
    }
}

pub fn format_invalid_type_assignability(
    f: &mut fmt::Formatter,
    indentation: usize,
    assignee: &Type,
    assigned: &Type,
    because: &Vec<TypeAssignability>,
    invariant: bool,
) -> fmt::Result {
    if indentation > 0 {
        write!(f, "\n")?;
    }

    for _ in 0..indentation {
        write!(f, "  ")?;
    }

    if indentation > 0 {
        write!(f, "because ")?;
    }

    if invariant {
        write!(f, "`{}` isn't the same as `{}`", assigned, assignee)?;
    } else {
        write!(f, "`{}` cannot act as `{}`", assigned, assignee)?;
    }

    for b in because.iter() {
        format_type_assignability(f, indentation + 1, b)?;
    }

    Ok(())
}

fn format_type_assignability(
    f: &mut fmt::Formatter,
    mut indentation: usize,
    assignability: &TypeAssignability,
) -> fmt::Result {
    match assignability {
        TypeAssignability::Valid => Ok(()),
        TypeAssignability::Uncoercable {
            from,
            to,
            already_coerced_to,
            because,
        } => {
            for _ in 0..indentation {
                write!(f, "  ")?;
            }

            if indentation > 0 {
                write!(f, "because ")?;
            }

            write!(f, "`{}` cannot be coerced into `{}`", from, to)?;

            if let Some(at) = already_coerced_to {
                write!(f, "\n")?;
                indentation += 1;
                for _ in 0..indentation {
                    write!(f, "  ")?;
                }
                write!(f, "because it's already coerced to `{}` and", at)?;
            }

            for b in because.iter() {
                format_type_assignability(f, indentation + 1, b)?;
            }

            Ok(())
        }
        TypeAssignability::Invalid {
            assignee,
            assigned,
            because,
            invariant,
        } => format_invalid_type_assignability(
            f,
            indentation,
            assignee,
            assigned,
            because,
            *invariant,
        ),
    }
}

impl fmt::Display for TypeAssignability {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format_type_assignability(f, 0, self)?;
        write!(f, ".")
    }
}

fn resolve_number(
    unresolved: &Type,
    proposed: &Type,
    analysis: &mut Analysis,
) -> Option<TypeAssignability> {
    if let Type::Class(_, class, _) = proposed {
        let class = analysis.navigator.find_node(*class)?;
        let (name, _, _) = analysis.navigator.qualified_name_of(&class)?;

        match (unresolved, name.as_ref()) {
            (Type::UnresolvedFloat(_, id), "Loa/Number")
            | (Type::UnresolvedFloat(_, id), "Loa/Float")
            | (Type::UnresolvedFloat(_, id), "Loa/Float16")
            | (Type::UnresolvedFloat(_, id), "Loa/Float32")
            | (Type::UnresolvedFloat(_, id), "Loa/Float64")
            | (Type::UnresolvedFloat(_, id), "Loa/BigFloat")
            | (Type::UnresolvedInteger(_, id), "Loa/Number")
            | (Type::UnresolvedInteger(_, id), "Loa/Float")
            | (Type::UnresolvedInteger(_, id), "Loa/Float16")
            | (Type::UnresolvedInteger(_, id), "Loa/Float32")
            | (Type::UnresolvedInteger(_, id), "Loa/Float64")
            | (Type::UnresolvedInteger(_, id), "Loa/BigFloat")
            | (Type::UnresolvedInteger(_, id), "Loa/Integer")
            | (Type::UnresolvedInteger(_, id), "Loa/Natural")
            | (Type::UnresolvedInteger(_, id), "Loa/Int8")
            | (Type::UnresolvedInteger(_, id), "Loa/Int16")
            | (Type::UnresolvedInteger(_, id), "Loa/Int32")
            | (Type::UnresolvedInteger(_, id), "Loa/Int64")
            | (Type::UnresolvedInteger(_, id), "Loa/BigInteger")
            | (Type::UnresolvedInteger(_, id), "Loa/UInt8")
            | (Type::UnresolvedInteger(_, id), "Loa/UInt16")
            | (Type::UnresolvedInteger(_, id), "Loa/UInt32")
            | (Type::UnresolvedInteger(_, id), "Loa/UInt64")
            | (Type::UnresolvedInteger(_, id), "Loa/BigNatural") => {
                if let Some(existing) = analysis.types.attempt_type_coercion(*id, proposed) {
                    Some(
                        check_assignment(existing.clone(), proposed.clone(), analysis, false)
                            .expect(|because| TypeAssignability::Uncoercable {
                                from: unresolved.clone(),
                                to: proposed.clone(),
                                already_coerced_to: Some(existing),
                                because,
                            }),
                    )
                } else {
                    Some(TypeAssignability::Valid)
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

pub fn check_assignment(
    assignee: Type,
    assigned: Type,
    analysis: &mut Analysis,
    invariant: bool,
) -> TypeAssignability {
    match (&assignee, &assigned) {
        (Type::Unknown, _) | (_, Type::Unknown) => TypeAssignability::Valid,

        // TODO: Implement type coercion
        (u @ Type::UnresolvedInteger(_, _), p)
        | (p, u @ Type::UnresolvedInteger(_, _))
        | (u @ Type::UnresolvedFloat(_, _), p)
        | (p, u @ Type::UnresolvedFloat(_, _)) => resolve_number(u, p, analysis).unwrap_or(
            TypeAssignability::Invalid {
                assignee: assignee.clone(),
                assigned: assigned.clone(),
                invariant,
                because: vec![],
            }
            .expect(|because| TypeAssignability::Uncoercable {
                from: u.clone(),
                to: p.clone(),
                already_coerced_to: None,
                because,
            }),
        ),

        (Type::Class(_, _, _), Type::Self_(box of)) => {
            return check_assignment(assignee.clone(), of.clone(), analysis, invariant).expect(
                |because| TypeAssignability::Invalid {
                    assignee,
                    assigned,
                    invariant,
                    because,
                },
            );
        }

        (Type::Self_(_), Type::Self_(_)) => {
            // A self is always assignable to a self, because they should onle
            // be comparable in the same class hierarchy.
            TypeAssignability::Valid
        }

        (Type::Self_(_), _) => TypeAssignability::Invalid {
            assignee,
            assigned,
            invariant,
            because: vec![],
        },

        (
            Type::Class(_, assignee_class, assignee_args),
            Type::Class(_, assigned_class, assigned_args),
        ) => {
            let assignee_class = analysis.navigator.find_node(*assignee_class)?;
            let assigned_class = analysis.navigator.find_node(*assigned_class)?;

            if assignee_class.id != assigned_class.id {
                if !invariant {
                    for super_type in analysis.navigator.super_type_expressions(&assigned_class) {
                        let super_type = analysis.types.get_type_of_type_expression(&super_type);

                        let assigned_super_type_assignability =
                            check_assignment(assignee.clone(), super_type, analysis, false);

                        if assigned_super_type_assignability.is_valid() {
                            return assigned_super_type_assignability;
                        }
                    }
                }

                return TypeAssignability::Invalid {
                    assigned,
                    assignee,
                    invariant,
                    because: vec![],
                };
            }

            if let Class {
                type_parameter_list,
                ..
            } = assignee_class.kind
            {
                let type_parameter_list = analysis
                    .navigator
                    .find_child(&assignee_class, type_parameter_list)?;
                if let TypeParameterList {
                    ref type_parameters,
                    ..
                } = type_parameter_list.kind
                {
                    let mut issues = vec![];

                    for (i, parameter) in type_parameters.iter().enumerate() {
                        let assignee_arg = if assignee_args.len() > i {
                            assignee_args[i].clone()
                        } else {
                            Type::Unknown
                        };

                        let assigned_arg = if assigned_args.len() > i {
                            assigned_args[i].clone()
                        } else {
                            Type::Unknown
                        };

                        let parameter = analysis
                            .navigator
                            .find_child(&type_parameter_list, *parameter)?;
                        if let TypeParameter {
                            variance_keyword, ..
                        } = parameter.kind
                        {
                            let arg_assignability = match variance_keyword.map(|t| t.kind) {
                                None | Some(TokenKind::InoutKeyword) => {
                                    check_assignment(assignee_arg, assigned_arg, analysis, true)
                                }
                                Some(TokenKind::OutKeyword) => {
                                    check_assignment(assignee_arg, assigned_arg, analysis, false)
                                }
                                Some(TokenKind::InKeyword) => {
                                    check_assignment(assigned_arg, assignee_arg, analysis, false)
                                }
                                _ => TypeAssignability::Valid,
                            };

                            if arg_assignability.is_invalid() {
                                issues.push(arg_assignability);
                            }
                        }
                    }

                    if issues.len() > 0 {
                        return TypeAssignability::Invalid {
                            assigned,
                            assignee,
                            invariant,
                            because: issues,
                        };
                    }
                }
            }
            TypeAssignability::Valid
        }

        // TODO: Implement constraints on type parameters
        // which would be checked here
        (Type::Parameter(_, _, _), _) => TypeAssignability::Valid,
        (Type::Class(_, _, _), Type::Parameter(_, _, _)) => TypeAssignability::Invalid {
            assigned,
            assignee,
            invariant,
            because: vec![],
        },

        (Type::Behaviour(box assignee_behaviour), Type::Behaviour(box assigned_behaviour)) => {
            if assignee_behaviour.selector() != assigned_behaviour.selector() {
                return TypeAssignability::Invalid {
                    assigned,
                    assignee,
                    invariant,
                    because: vec![],
                };
            }

            let mut issues = vec![];

            let return_type_assignability = check_assignment(
                assignee_behaviour.return_type.clone(),
                assigned_behaviour.return_type.clone(),
                analysis,
                invariant,
            );
            if return_type_assignability.is_invalid() {
                issues.push(return_type_assignability);
            }

            match (&assignee_behaviour.message, &assigned_behaviour.message) {
                (
                    BehaviourMessage::Binary(_, ref assignee_arg),
                    BehaviourMessage::Binary(_, ref assigned_arg),
                ) => {
                    check_message_argument(
                        assignee_arg.clone(),
                        assigned_arg.clone(),
                        analysis,
                        &mut issues,
                    );
                }
                (
                    BehaviourMessage::Keyword(ref assignee_kws),
                    BehaviourMessage::Keyword(ref assigned_kws),
                ) => {
                    for ((_, assignee_arg), (_, assigned_arg)) in
                        assignee_kws.iter().zip(assigned_kws.iter())
                    {
                        check_message_argument(
                            assignee_arg.clone(),
                            assigned_arg.clone(),
                            analysis,
                            &mut issues,
                        );
                    }
                }
                _ => {}
            }

            if issues.len() == 0 {
                TypeAssignability::Valid
            } else {
                TypeAssignability::Invalid {
                    assigned,
                    assignee,
                    invariant,
                    because: issues,
                }
            }
        }

        (_, Type::Behaviour(_)) | (Type::Behaviour(_), _) => TypeAssignability::Invalid {
            assigned,
            assignee,
            invariant,
            because: vec![],
        },
    }
}

fn check_message_argument(
    assignee_arg: Type,
    assigned_arg: Type,
    analysis: &mut Analysis,
    issues: &mut Vec<TypeAssignability>,
) {
    let assignment = check_assignment(assigned_arg.clone(), assignee_arg.clone(), analysis, false);
    if assignment.is_invalid() {
        issues.push(assignment);
    }
}
