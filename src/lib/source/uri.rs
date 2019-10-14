use std::fmt;
use std::path::PathBuf;

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum URI {
    #[cfg(test)]
    Test,

    Exact(String),
    File(PathBuf),
    Stdin,
}

impl fmt::Display for URI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(test)]
            URI::Test => write!(f, "test:"),

            URI::Exact(s) => write!(f, "{}", s),
            URI::File(path) => write!(f, "file://{}", path.display()),
            URI::Stdin => write!(f, "stdin:"),
        }
    }
}
