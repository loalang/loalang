#![feature(box_patterns)]

pub use std::borrow::Cow;
pub use std::collections::HashMap;
pub use std::collections::HashSet;
pub use std::fmt;
pub use std::slice::Iter;
pub use std::sync::Arc;

extern crate matches;

use matches::*;

extern crate num_bigint;

use num_bigint::BigInt;

extern crate glob;

use glob::glob;

mod source;

pub use self::source::*;

#[macro_use]
mod diagnostics;

pub use self::diagnostics::*;

pub mod syntax;

pub mod semantics;

pub mod format;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::{tokenize, Parser};
    use std::ffi::OsStr;
    use std::io;
    use std::path::Component;

    #[test]
    fn fixtures() -> io::Result<()> {
        let modules = Source::files("src/__fixtures__/**/*.loa")?;
        let mut projects = HashMap::<String, Vec<Arc<Source>>>::new();
        for module in modules {
            if let URI::File(path) = module.uri.clone() {
                let mut components = path.components();
                let fixture = OsStr::new("__fixtures__");
                loop {
                    match components.next() {
                        Some(Component::Normal(s)) if s == fixture => {
                            break;
                        }

                        None => {
                            panic!("Outside fixtures dir");
                        }

                        _ => {}
                    }
                }

                let fixture_namespace = components
                    .next()
                    .unwrap()
                    .as_os_str()
                    .to_str()
                    .unwrap()
                    .to_string();

                if !projects.contains_key(&fixture_namespace) {
                    projects.insert(fixture_namespace.clone(), vec![]);
                }

                projects.get_mut(&fixture_namespace).unwrap().push(module);
            }
        }

        for (project, v) in projects {
            let mut modules = vec![];
            let mut comments = vec![];
            for s in v {
                let tokens = tokenize(s)
                    .into_iter()
                    .filter_map(|t| match t.kind {
                        syntax::TokenKind::Whitespace(_) => None,
                        syntax::TokenKind::LineComment(_) => {
                            comments.push(t);
                            None
                        }
                        _ => Some(t),
                    })
                    .collect();
                let mut p = Parser::from_tokens(tokens);
                modules.push(p.parse_module());
            }
            Diagnosed::extract_flat_map(modules, |m| m)
                .map(|m| semantics::Resolver::new().resolve_modules(&m))
                .flat_map(|p| {
                    let mut global_scope = semantics::LexicalScope::new();
                    global_scope.register_program(&p);
                    global_scope.resolve_program(p)
                })
                .flat_map(|p| {
                    let mut resolver = semantics::TypeResolver::new();
                    resolver.resolve_program(&p);
                    Diagnosed::Diagnosis(p, resolver.diagnostics)
                })
                .diagnostics(|diagnostics| {
                    fn diagnostic_matches_comment(
                        diagnostic: &Diagnostic,
                        comment: &syntax::Token,
                    ) -> bool {
                        if let syntax::TokenKind::LineComment(ref c) = comment.kind {
                            diagnostic.span().unwrap().start.line == comment.span.start.line
                                && diagnostic.to_string() == c.trim()
                        } else {
                            false
                        }
                    }

                    'diagnostics: for diagnostic in diagnostics.iter() {
                        for comment in comments.iter() {
                            if diagnostic_matches_comment(diagnostic, comment) {
                                continue 'diagnostics;
                            }
                        }
                        let span = diagnostic.span().unwrap();
                        assert!(
                            false,
                            "[{}] Unexpected diagnostic: \"{:?}\" at {}:{}",
                            project, diagnostic, span.start.source.uri, span.start.line
                        );
                    }
                    'comments: for comment in comments.iter() {
                        for diagnostic in diagnostics.iter() {
                            if diagnostic_matches_comment(diagnostic, comment) {
                                continue 'comments;
                            }
                        }
                        assert!(
                            false,
                            "[{}] Expected diagnostic: \"{}\" at {}:{}",
                            project,
                            &comment.lexeme()[1..],
                            comment.span.start.source.uri,
                            comment.span.start.line
                        );
                    }
                });
        }

        Ok(())
    }
}
