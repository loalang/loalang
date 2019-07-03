use std::path::PathBuf;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum URI {
    #[cfg(test)]
    Test,

    File(PathBuf),
    Stdin,
}

impl fmt::Display for URI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(test)]
            URI::Test => write!(f, "test:"),

            URI::File(path) => write!(f, "file://{}", path.display()),
            URI::Stdin => write!(f, "stdin:"),
        }
    }
}
