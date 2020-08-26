use std::error::Error as StdError;
use std::fmt;
use std::result;

use thiserror::Error;

/// A result type to use throughout this crate.
pub type Result<T> = result::Result<T, Error>;

/// The kind of error that occurred.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The radiotap header has unsupported version.
    UnsupportedVersion(u8),
    /// Failed to parse the radiotap header.
    Header,
    /// Failed to skip a field.
    Skip,
    /// Failed to read a field.
    Read(&'static str),
}

/// An error that can occur in this crate.
#[derive(Debug, Error)]
#[error("{}", self.kind)]
pub struct Error {
    kind: ErrorKind,
    #[source]
    source: Option<Box<dyn StdError + Send + Sync>>,
}

pub(crate) trait ResultExt<T> {
    fn context(self, kind: ErrorKind) -> Result<T>;
    fn with_context<F: FnOnce() -> ErrorKind>(self, f: F) -> Result<T>;
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported radiotap version `{}`", version)
            }
            Self::Header => write!(f, "failed to parse the radiotap header"),
            Self::Skip => write!(f, "failed to skip field"),
            Self::Read(ty) => write!(f, "failed to read field `{}`", ty),
        }
    }
}

impl Error {
    /// Create a new `Error`.
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self { kind, source: None }
    }

    /// Returns the kind of error.
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }
}

impl<T, E> ResultExt<T> for result::Result<T, E>
where
    E: Into<Box<dyn StdError + Send + Sync>>,
{
    fn context(self, kind: ErrorKind) -> Result<T> {
        self.map_err(|e| Error {
            kind,
            source: Some(e.into()),
        })
    }

    fn with_context<F: FnOnce() -> ErrorKind>(self, f: F) -> Result<T> {
        self.map_err(|e| Error {
            kind: f(),
            source: Some(e.into()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_custom_source() {
        #[derive(Debug, Error)]
        #[error("test")]
        struct MyError;

        let result: Result<&str> = Err(Error {
            kind: ErrorKind::Skip,
            source: Some(Box::new(MyError)),
        });
        let error = anyhow::Context::context(result, "context").unwrap_err();

        assert_eq!(
            format!("{:?}", error),
            "context\n\nCaused by:\n    0: failed to skip field\n    1: test"
        );
    }
}
