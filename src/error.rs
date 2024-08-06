use std::path::PathBuf;

use thiserror::Error;

pub type Result<A> = std::result::Result<A, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Encountered io error: `{0}`")]
    IOError(std::io::Error),
    #[error("Unexpected path for a page: `{0}`")]
    PageError(PathBuf),
    #[error("Unexpected path for a directory: `{0}`")]
    DirError(PathBuf),
    #[error("Failed to parse yaml: `{0}`")]
    SerdeError(serde_yml::Error),
    #[error("Error with templating: `{0}`")]
    JinjaError(minijinja::Error),
    #[error("Error watching files: `{0}`")]
    NotifyError(notify::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IOError(value)
    }
}

impl From<serde_yml::Error> for Error {
    fn from(value: serde_yml::Error) -> Self {
        Self::SerdeError(value)
    }
}
impl From<minijinja::Error> for Error {
    fn from(value: minijinja::Error) -> Self {
        Self::JinjaError(value)
    }
}
impl From<notify::Error> for Error {
    fn from(value: notify::Error) -> Self {
        Self::NotifyError(value)
    }
}
