
/// `Error` is the error type returned for failure in deserialzation process
#[derive(Debug)]
pub struct Error (ErrorKind);

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ErrorKind {
    IncorrectLength,
    ReceiptStatusCodeOutOfRange,
    StringParseError,
}

impl Error {
    pub fn new(errorkind :ErrorKind) -> Self {
        Self(errorkind)
    }

    pub fn kind(&self) -> ErrorKind {
        self.0
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "Deserialization error"
    }
}
