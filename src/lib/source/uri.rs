use std::fmt;
use std::path::PathBuf;

#[derive(Eq, Debug, Clone, Hash)]
pub enum URI {
    #[cfg(test)]
    Test,

    Exact(String),
    File(PathBuf),
    Stdin,
    REPLLine(usize),

    Main,
}

impl URI {
    pub fn neighboring_file(&self, name: &str) -> Option<URI> {
        match self {
            URI::Exact(s) => {
                let mut segments: Vec<_> = s.split("/").collect();
                segments.pop();
                segments.push(name.as_ref());
                Some(URI::Exact(segments.join("/")))
            }
            URI::File(path) => {
                let mut path = path.clone();
                path.pop();
                Some(URI::File(path))
            }
            _ => None,
        }
    }

    pub fn basename(&self) -> Option<String> {
        match self {
            URI::Exact(s) => {
                let mut segments: Vec<_> = s.split("/").collect();
                segments.pop().map(|s| s.into())
            }
            URI::File(path) => path.file_name().map(|os| os.to_string_lossy().to_string()),
            _ => None,
        }
    }

    pub fn matches_basename(&self, basename: &str) -> bool {
        self.basename() == Some(basename.into())
    }
}

impl PartialEq for URI {
    fn eq(&self, other: &Self) -> bool {
        format!("{}", self) == format!("{}", other)
    }
}

impl fmt::Display for URI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(test)]
            URI::Test => write!(f, "test:"),

            URI::Exact(s) => write!(f, "{}", s),
            URI::File(path) => write!(f, "file://{}", path.display()),
            URI::Stdin => write!(f, "stdin:"),
            URI::REPLLine(n) => write!(f, "repl:{}", n),

            URI::Main => write!(f, "main"),
        }
    }
}
