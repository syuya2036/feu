#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("Route not found")]
    NotFound,
    #[error("Internal server error")]
    Internal,
    #[error("Bad request")]
    BadRequest,
    #[error("{0}")]
    Message(String),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error {
    #[from]
    kind: ErrorKind,
}

impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn msg(msg: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Message(msg.into()),
        }
    }
}

// Allow conversion from anyhow or other error types later if needed,
// for now keeping it minimal.

pub type Result<T> = std::result::Result<T, Error>;
