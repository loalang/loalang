use crate::*;
use std::fmt;
use std::io::{self, Read};
use std::path::PathBuf;

pub struct Source {
    pub kind: SourceKind,
    pub uri: URI,
    pub code: String,
}

#[derive(Clone)]
pub enum SourceKind {
    Module,
    REPLLine,
}

impl Source {
    pub fn new(kind: SourceKind, uri: URI, code: String) -> Arc<Source> {
        Arc::new(Source { kind, uri, code })
    }

    pub fn file(path: PathBuf) -> io::Result<Arc<Source>> {
        let uri = URI::File(path.clone());
        Self::file_with_uri(path, uri)
    }

    pub fn stdin() -> io::Result<Arc<Source>> {
        let mut code = String::new();
        io::stdin().read_to_string(&mut code)?;
        Ok(Self::new(SourceKind::Module, URI::Stdin, code))
    }

    pub fn files<S: AsRef<str>>(s: S) -> io::Result<Vec<Arc<Source>>> {
        Self::files_with_uri(s.as_ref(), |path| URI::File(path))
    }

    fn files_with_uri<F: Fn(PathBuf) -> URI>(g: &str, f: F) -> io::Result<Vec<Arc<Source>>> {
        let mut sources = vec![];
        match glob::glob(g) {
            Ok(paths) => {
                for path in paths {
                    if let Ok(path) = path {
                        let uri = f(path.clone());
                        sources.push(Self::file_with_uri(path, uri)?);
                    }
                }
            }
            _ => (),
        }
        Ok(sources)
    }

    fn file_with_uri(path: PathBuf, uri: URI) -> io::Result<Arc<Source>> {
        let path = path.canonicalize()?;
        Ok(Self::new(
            SourceKind::Module,
            uri,
            std::fs::read_to_string(path)?,
        ))
    }

    pub fn stdlib() -> io::Result<Vec<Arc<Source>>> {
        Self::files_with_uri("/usr/local/lib/loa/std/**/*.loa", |path| URI::Stdlib(path))
    }

    #[cfg(test)]
    pub fn test(code: &str) -> Arc<Source> {
        Self::new(SourceKind::Module, URI::Test, code.into())
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}

impl fmt::Debug for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Source({})", self.uri)
    }
}
