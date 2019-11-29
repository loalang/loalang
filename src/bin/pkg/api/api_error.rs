use loa::*;

pub type APIResult<T> = Result<T, APIError>;

#[derive(Debug)]
pub enum APIError {
    Http(reqwest::Error),
    Serde(serde_json::Error),
    GraphQL(Option<Vec<graphql_client::Error>>),
    IO(std::io::Error),
    InvalidCredentials,
    PackageNotFound,
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
        APIError::Serde(err)
    }
}

impl From<std::io::Error> for APIError {
    fn from(err: std::io::Error) -> Self {
        APIError::IO(err)
    }
}
