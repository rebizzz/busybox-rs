use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum BbError {
    NotFound { path: PathBuf },
    PermissionDenied { path: PathBuf },
    IsDirectory { path: PathBuf },
    NotDirectory { path: PathBuf },
    AlreadyExists { path: PathBuf },
    Io { path: Option<PathBuf>, source: io::Error },
    Custom(String),
}

impl fmt::Display for BbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BbError::NotFound { path } => write!(f, "{}: No such file or directory", path.display()),
            BbError::PermissionDenied { path } => write!(f, "{}: Permission denied", path.display()),
            BbError::IsDirectory { path } => write!(f, "{}: Is a directory", path.display()),
            BbError::NotDirectory { path } => write!(f, "{}: Not a directory", path.display()),
            BbError::AlreadyExists { path } => write!(f, "{}: File exists", path.display()),
            BbError::Io { path: Some(p), source } => write!(f, "{}: {}", p.display(), source),
            BbError::Io { path: None, source } => write!(f, "{}", source),
            BbError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BbError {}

impl From<io::Error> for BbError {
    fn from(err: io::Error) -> Self {
        BbError::Io { path: None, source: err }
    }
}
