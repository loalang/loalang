use crate::*;
use std::f64::INFINITY;
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
    InvalidLiteralType(Span, semantics::Type),
    OutOfBounds(Span, semantics::Type, String),
    TooPreciseFloat(Span, semantics::Type, BigFraction),
    WrongNumberOfTypeArguments(Span, String, usize, usize),
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
            InvalidLiteralType(ref s, _) => s,
            OutOfBounds(ref s, _, _) => s,
            TooPreciseFloat(ref s, _, _) => s,
            WrongNumberOfTypeArguments(ref s, _, _, _) => s,
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
            InvalidLiteralType(_, _) => DiagnosticLevel::Error,
            OutOfBounds(_, _, _) => DiagnosticLevel::Error,
            TooPreciseFloat(_, _, _) => DiagnosticLevel::Warning,
            WrongNumberOfTypeArguments(_, _, _, _) => DiagnosticLevel::Error,
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
            InvalidLiteralType(_, _) => 10,
            OutOfBounds(_, _, _) => 11,
            TooPreciseFloat(_, _, _) => 12,
            WrongNumberOfTypeArguments(_, _, _, _) => 13,
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
            InvalidLiteralType(_, type_) => {
                write!(f, "`{}` is not a valid type for this literal.", type_)
            }
            OutOfBounds(_, type_, message) => write!(f, "`{}` must not be {}.", type_, message),
            TooPreciseFloat(_, type_, fraction) => write!(
                f,
                "`{:.2$}` is too precise to be coerced to {} without losing precision.",
                fraction, type_, INFINITY as usize
            ),
            WrongNumberOfTypeArguments(_, name, params, args) => write!(
                f,
                "`{}` takes {} type arguments, but was provided {}.",
                name,
                if *params == 0 {
                    "no".into()
                } else {
                    params.to_string()
                },
                if *args == 0 {
                    "none".into()
                } else {
                    args.to_string()
                },
            ),
        }
    }
}

pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}
