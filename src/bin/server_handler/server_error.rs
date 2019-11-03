use loa::*;

#[derive(Debug)]
pub enum ServerError {
    Empty,
    SerializationFailure,
}

impl ServerError {
    pub fn code(&self) -> i32 {
        match self {
            ServerError::Empty => -1,
            ServerError::SerializationFailure => -2,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ServerError::Empty => "No response available.".into(),
            ServerError::SerializationFailure => "Failed (de)serialization.".into(),
        }
    }
}

impl Error for ServerError {}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl From<std::option::NoneError> for ServerError {
    fn from(_: std::option::NoneError) -> Self {
        ServerError::Empty
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(_: serde_json::Error) -> Self {
        ServerError::SerializationFailure
    }
}
