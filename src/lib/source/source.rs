use crate::*;
use std::path::PathBuf;
use std::io;
use std::fmt;

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
        Ok(Self::new(URI::File(path.clone()), std::fs::read_to_string(path)?))
    }

    #[cfg(test)]
    pub fn test(code: String) -> Arc<Source> {
        Self::new(URI::Test, code)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.uri)
    }
}
