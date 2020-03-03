use crate::*;
use std::f64::INFINITY;
use std::fmt;

#[derive(Clone, IntoStaticStr)]
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
    InvalidAccessToPrivateMethod(Span, String, String),
    InvalidTypeParameterReferenceVarianceUsage(Span, String, &'static str, &'static str),
    IncompleteInitializer(Span, String, Vec<String>),
    UndefinedInitializedVariable(Span, String, String),
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
            SyntaxError(ref s, _)
            | UndefinedTypeReference(ref s, _)
            | UndefinedReference(ref s, _)
            | UndefinedBehaviour(ref s, _, _)
            | UndefinedImport(ref s, _)
            | UnexportedImport(ref s, _)
            | UnassignableType { span: ref s, .. }
            | DuplicatedDeclaration(ref s, _, _)
            | InvalidInherit { span: ref s, .. }
            | InvalidLiteralType(ref s, _)
            | OutOfBounds(ref s, _, _)
            | TooPreciseFloat(ref s, _, _)
            | WrongNumberOfTypeArguments(ref s, _, _, _)
            | InvalidAccessToPrivateMethod(ref s, _, _)
            | InvalidTypeParameterReferenceVarianceUsage(ref s, _, _, _)
            | IncompleteInitializer(ref s, _, _)
            | UndefinedInitializedVariable(ref s, _, _) => s,
        }
    }

    pub fn level(&self) -> DiagnosticLevel {
        use Diagnostic::*;

        match self {
            SyntaxError(_, _)
            | UndefinedTypeReference(_, _)
            | UndefinedReference(_, _)
            | UndefinedBehaviour(_, _, _)
            | UndefinedImport(_, _)
            | UnexportedImport(_, _)
            | UnassignableType { .. }
            | DuplicatedDeclaration(_, _, _)
            | InvalidInherit { .. }
            | InvalidLiteralType(_, _)
            | OutOfBounds(_, _, _)
            | WrongNumberOfTypeArguments(_, _, _, _)
            | InvalidAccessToPrivateMethod(_, _, _)
            | InvalidTypeParameterReferenceVarianceUsage(_, _, _, _)
            | IncompleteInitializer(_, _, _)
            | UndefinedInitializedVariable(_, _, _) => DiagnosticLevel::Error,

            TooPreciseFloat(_, _, _) => DiagnosticLevel::Warning,
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
            InvalidAccessToPrivateMethod(_, _, _) => 14,
            InvalidTypeParameterReferenceVarianceUsage(_, _, _, _) => 15,
            IncompleteInitializer(_, _, _) => 16,
            UndefinedInitializedVariable(_, _, _) => 17,
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

impl fmt::Debug for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name: &'static str = self.into();
        write!(
            f,
            "{:?} ({} @ {}:{})",
            self.to_string(),
            name,
            self.span().start.uri,
            self.span().start.line,
        )
    }
}

impl fmt::Display for Diagnostic {
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
            InvalidAccessToPrivateMethod(_, class_name, method_selector) => {
                write!(f, "`{}#{}` is private.", class_name, method_selector)
            }
            InvalidTypeParameterReferenceVarianceUsage(_, name, usage, mark) => write!(
                f,
                "`{}` cannot be used in {} position, because it's marked as `{}`.",
                name, usage, mark
            ),
            IncompleteInitializer(_, selector, uninitialized_names) => {
                write!(f, "Initializer `{}` must initialize ", selector)?;

                match uninitialized_names.len() {
                    1 => write!(f, "`{}`.", &uninitialized_names[0]),
                    2 => write!(
                        f,
                        "`{}` and `{}`.",
                        &uninitialized_names[0], &uninitialized_names[1]
                    ),
                    n => {
                        for (i, name) in uninitialized_names.iter().enumerate() {
                            if i < n - 1 {
                                write!(f, "`{}`, ", name)?;
                            } else {
                                write!(f, "and `{}`", name)?;
                            }
                        }
                        write!(f, ".")
                    }
                }
            }
            UndefinedInitializedVariable(_, var_name, class_name) => {
                write!(f, "`{}` is not a variable of `{}`.", var_name, class_name)
            }
        }
    }
}

pub enum DiagnosticLevel {
    Error,
    Warning,
    Info,
}
