use loa::*;

pub type APIResult<T> = Result<T, APIError>;

#[derive(Debug)]
pub enum APIError {
    Http(reqwest::Error),
    SerdeJSON(serde_json::Error),
    SerdeYAML(serde_yaml::Error),
    GraphQL(Option<Vec<graphql_client::Error>>),
    IO(std::io::Error),
    Glob(glob::GlobError),
    Ignore(ignore::Error),
    InvalidCredentials,
    PackageNotFound,
    ChecksumMismatch,
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<reqwest::Error> for APIError {
    fn from(err: reqwest::Error) -> Self {
        APIError::Http(err)
    }
}

impl From<serde_json::Error> for APIError {
    fn from(err: serde_json::Error) -> Self {
        APIError::SerdeJSON(err)
    }
}

impl From<serde_yaml::Error> for APIError {
    fn from(err: serde_yaml::Error) -> Self {
        APIError::SerdeYAML(err)
    }
}
impl From<glob::GlobError> for APIError {
    fn from(err: glob::GlobError) -> Self {
        APIError::Glob(err)
    }
}

impl From<std::io::Error> for APIError {
    fn from(err: std::io::Error) -> Self {
        APIError::IO(err)
    }
}

impl From<ignore::Error> for APIError {
    fn from(err: ignore::Error) -> Self {
        APIError::Ignore(err)
    }
}
