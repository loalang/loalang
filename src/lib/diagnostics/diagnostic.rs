use crate::*;
use std::fmt;

#[derive(Clone)]
pub enum Diagnostic {
    SyntaxError(Span, String),
    UndefinedTypeReference(Span, String),
    UndefinedReference(Span, String),
    UndefinedBehaviour(Span, semantics::Type, String),
    UndefinedImport(Span, String),
    UnexportedImport(Span, String),
    UnassignableType {
        span: Span,
        assignability: semantics::TypeAssignability,
    },
    DuplicatedDeclaration(Span, String, usize),
    InvalidInherit {
        span: Span,
        super_type: semantics::Type,
        sub_type: semantics::Type,
        violations: Vec<InheritanceViolation>,
    },
}

#[derive(Clone)]
pub enum InheritanceViolation {
    BehaviourNotImplemented(semantics::Behaviour),
    OverrideNotSound(semantics::Behaviour, semantics::TypeAssignability),
}

impl Diagnostic {
    pub fn span(&self) -> &Span {
        use Diagnostic::*;

        match self {
            SyntaxError(ref s, _) => s,
            UndefinedTypeReference(ref s, _) => s,
            UndefinedReference(ref s, _) => s,
            UndefinedBehaviour(ref s, _, _) => s,
            UndefinedImport(ref s, _) => s,
            UnexportedImport(ref s, _) => s,
            UnassignableType { span: ref s, .. } => s,
            DuplicatedDeclaration(ref s, _, _) => s,
            InvalidInherit { span: ref s, .. } => s,
        }
    }

    pub fn level(&self) -> DiagnosticLevel {
        use Diagnostic::*;

        match self {
            SyntaxError(_, _) => DiagnosticLevel::Error,
            UndefinedTypeReference(_, _) => DiagnosticLevel::Error,
            UndefinedReference(_, _) => DiagnosticLevel::Error,
            UndefinedBehaviour(_, _, _) => DiagnosticLevel::Error,
            UndefinedImport(_, _) => DiagnosticLevel::Error,
            UnexportedImport(_, _) => DiagnosticLevel::Error,
            UnassignableType { .. } => DiagnosticLevel::Error,
            DuplicatedDeclaration(_, _, _) => DiagnosticLevel::Error,
            InvalidInherit { .. } => DiagnosticLevel::Error,
        }
    }

    pub fn code(&self) -> usize {
        use Diagnostic::*;

        match self {
            SyntaxError(_, _) => 1,
            UndefinedTypeReference(_, _) => 2,
            UndefinedReference(_, _) => 3,
            UndefinedBehaviour(_, _, _) => 4,
            UndefinedImport(_, _) => 5,
            UnexportedImport(_, _) => 6,
            UnassignableType { .. } => 7,
            DuplicatedDeclaration(_, _, _) => 8,
            InvalidInherit { .. } => 9,
        }
    }

    pub fn failed(diagnostics: &Vec<Diagnostic>) -> bool {
        let mut failed = false;
        for diagnostic in diagnostics.iter() {
            if let DiagnosticLevel::Error = diagnostic.level() {
                failed = true;
            }
        }
        failed
    }
}

impl ToString for Diagnostic {
    fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Diagnostic::*;

        match self {
            SyntaxError(_, s) => write!(f, "{}", s),
            UndefinedTypeReference(_, s) => write!(f, "`{}` is undefined.", s),
            UndefinedReference(_, s) => write!(f, "`{}` is undefined.", s),
            UndefinedBehaviour(_, t, s) => write!(f, "`{}` doesn't respond to `{}`.", t, s),
            UndefinedImport(_, s) => write!(f, "`{}` is undefined.", s),
            UnexportedImport(_, s) => write!(f, "`{}` is not exported.", s),
            UnassignableType { assignability, .. } => write!(f, "{}", assignability),
            DuplicatedDeclaration(_, s, n) => {
                write!(f, "`{}` is defined {} times in this scope.", s, n)
            }
            InvalidInherit {
                sub_type,
                super_type,
                violations,
                ..
            } => {
                write!(f, "`{}` doesn't act as `{}` because:", sub_type, super_type)?;
                for violation in violations.iter() {
                    match violation {
                        InheritanceViolation::BehaviourNotImplemented(ref b) => {
                            write!(f, "\n  - it doesn't respond to `{}`", b)?
                        }
                        InheritanceViolation::OverrideNotSound(ref b, ref t) => {
                            write!(
                                f,
                                "\n• it doesn't respond to `{}` like `{}` would",
                                b.selector(),
                                super_type,
                            )?;
                            if let semantics::TypeAssignability::Invalid {
                                assignee,
                                assigned,
                                because,
                                invariant,
                            } = t
                            {
                                semantics::format_invalid_type_assignability(
                                    f, 2, assignee, assigned, &because, *invariant,
                                )?;
                            }
                        }
                    }
                }
                Ok(())
            }
        }
    }
}

pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}
