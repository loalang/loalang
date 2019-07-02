use std::path::PathBuf;
use std::fmt;

pub enum URI {
    #[cfg(test)]
    Test,

    File(PathBuf),
}

impl fmt::Display for URI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            #[cfg(test)]
            URI::Test => write!(f, "test:"),

            URI::File(path) => write!(f, "file://{}", path.display()),
        }
    }
}
