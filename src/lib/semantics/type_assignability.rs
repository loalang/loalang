use crate::semantics::*;
use crate::*;

#[derive(Clone)]
pub enum TypeAssignability {
    Valid,
    Invalid {
        assignee: Type,
        assigned: Type,
        invariant: bool,
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
        if let TypeAssignability::Invalid { .. } = self {
            true
        } else {
            false
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

fn format_invalid_type_assignability(
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
    indentation: usize,
    assignability: &TypeAssignability,
) -> fmt::Result {
    match assignability {
        TypeAssignability::Valid => Ok(()),
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
