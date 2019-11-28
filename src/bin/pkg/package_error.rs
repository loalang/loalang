use loa::*;

#[derive(Debug)]
pub enum PackageError {
    Http(reqwest::Error),
    Serde(serde_json::Error),
    FailedToUpload(Option<Vec<graphql_client::Error>>),
}

impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<reqwest::Error> for PackageError {
    fn from(err: reqwest::Error) -> Self {
        PackageError::Http(err)
    }
}

impl From<serde_json::Error> for PackageError {
    fn from(err: serde_json::Error) -> Self {
        PackageError::Serde(err)
    }
}
