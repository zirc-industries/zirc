use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    pub msg: String,
    pub line: Option<usize>,
    pub col: Option<usize>,
}

impl Error {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            line: None,
            col: None,
        }
    }
    pub fn with_span(msg: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            msg: msg.into(),
            line: Some(line),
            col: Some(col),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let (Some(l), Some(c)) = (self.line, self.col) {
            write!(f, "{} at {}:{}", self.msg, l, c)
        } else {
            write!(f, "{}", self.msg)
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::new(s)
    }
}
impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::new(s)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn error<T>(msg: impl Into<String>) -> Result<T> {
    Err(Error::new(msg))
}

pub fn error_at<T>(line: usize, col: usize, msg: impl Into<String>) -> Result<T> {
    Err(Error::with_span(msg, line, col))
}
