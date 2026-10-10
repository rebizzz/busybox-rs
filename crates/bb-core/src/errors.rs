use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum BbError {
    NotFound {
        path: PathBuf,
    },
    PermissionDenied {
        path: PathBuf,
    },
    Io {
        path: Option<PathBuf>,
        source: io::Error,
    },
}

impl fmt::Display for BbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BbError::NotFound { path } => {
                write!(f, "{}: No such file or directory", path.display())
            }
            BbError::PermissionDenied { path } => {
                write!(f, "{}: Permission denied", path.display())
            }
            BbError::Io {
                path: Some(p),
                source,
            } => write!(f, "{}: {}", p.display(), source),
            BbError::Io { path: None, source } => write!(f, "{}", source),
        }
    }
}

impl std::error::Error for BbError {}

impl From<io::Error> for BbError {
    fn from(err: io::Error) -> Self {
        BbError::Io {
            path: None,
            source: err,
        }
    }
}
