use std::fmt;
use std::error::Error;
use std::io;

#[derive(Debug)]
pub enum ProcessCommunicationError {
    IoError(io::Error),
    CommandError(String),
    ThreadJoinError,
    MutexLockError,
    // Other(String),
}

impl fmt::Display for ProcessCommunicationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessCommunicationError::IoError(err) => write!(f, "I/O Error: {}", err),
            ProcessCommunicationError::CommandError(msg) => write!(f, "Command Error: {}", msg),
            ProcessCommunicationError::ThreadJoinError => write!(f, "Thread Join Error"),
            ProcessCommunicationError::MutexLockError => write!(f, "Mutex Lock Error"),
            // ProcessCommunicationError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl Error for ProcessCommunicationError {}

impl From<io::Error> for ProcessCommunicationError {
    fn from(err: io::Error) -> Self {
        ProcessCommunicationError::IoError(err)
    }
}
