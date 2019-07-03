use crate::*;
use std::fmt;
use std::io::{self, Read};
use std::path::PathBuf;

pub struct Source {
    pub uri: URI,
    pub code: String,
}

impl Source {
    pub fn new(uri: URI, code: String) -> Arc<Source> {
        Arc::new(Source { uri, code })
    }

    pub fn file(path: PathBuf) -> io::Result<Arc<Source>> {
        let path = path.canonicalize()?;
        Ok(Self::new(
            URI::File(path.clone()),
            std::fs::read_to_string(path)?,
        ))
    }

    pub fn stdin() -> io::Result<Arc<Source>> {
        let mut code = String::new();
        io::stdin().read_to_string(&mut code)?;
        Ok(Self::new(URI::Stdin, code))
    }

    #[cfg(test)]
    pub fn test(code: &str) -> Arc<Source> {
        Self::new(URI::Test, code.into())
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}
