use std::fmt;

pub type Result<T> = std::result::Result<T, NexoraError>;

#[derive(Debug)]
pub enum NexoraError {
    Engine(String),
    Storage(String),
    Navigation(String),
    Io(std::io::Error),
    Config(String),
    Win32(String),
    Serialization(String),
    Other(String),
}

impl fmt::Display for NexoraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Engine(msg) => write!(f, "Browser Engine Error: {}", msg),
            Self::Storage(msg) => write!(f, "Storage Error: {}", msg),
            Self::Navigation(msg) => write!(f, "Navigation Error: {}", msg),
            Self::Io(err) => write!(f, "I/O Error: {}", err),
            Self::Config(msg) => write!(f, "Configuration Error: {}", msg),
            Self::Win32(msg) => write!(f, "Windows System Error: {}", msg),
            Self::Serialization(msg) => write!(f, "Serialization Error: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for NexoraError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for NexoraError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for NexoraError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NexoraError::Engine("Initialization timeout".into());
        assert_eq!(format!("{}", err), "Browser Engine Error: Initialization timeout");

        let io_err = NexoraError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert!(format!("{}", io_err).contains("file not found"));
    }
}


/*
impl From<rusqlite::Error> for NexoraError {
    fn from(err: rusqlite::Error) -> Self {
        Self::Storage(err.to_string())
    }
}
*/
