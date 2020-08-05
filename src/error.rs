use std::error;
use std::fmt;
use std::io;
use std::str;

/// A stream parsing error.
#[derive(Debug)]
#[non_exhaustive]
pub enum ParseError {
    /// An I/O error occurred.
    Io(io::Error),
    /// A string decoding error occurred.
    Encoding(str::Utf8Error),
    /// The Ogg page was missing the `OggS` magic.
    InvalidOggPage,
    /// The Opus headers were missing.
    DidNotFindHeaders,
}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<str::Utf8Error> for ParseError {
    fn from(e: str::Utf8Error) -> Self {
        Self::Encoding(e)
    }
}

impl error::Error for ParseError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match self {
            ParseError::Io(e) => Some(e),
            ParseError::Encoding(e) => Some(e),
            ParseError::InvalidOggPage => None,
            ParseError::DidNotFindHeaders => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => e.fmt(f),
            ParseError::Encoding(e) => e.fmt(f),
            ParseError::InvalidOggPage => f.write_str("missing Ogg page magic"),
            ParseError::DidNotFindHeaders => f.write_str("missing Opus headers"),
        }
    }
}
