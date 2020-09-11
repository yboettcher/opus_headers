use std::error;
use std::fmt;
use std::io;
use std::str;
use std::result;

/// A specialized [`Result`][std::result::Result] type for the fallible functions.
pub type Result<T, E = ParseError> = result::Result<T, E>;

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
    /// The Opus headers was missing its magic number.
    InvalidOpusHeader,
    /// The Comment Header exceeds 120MB.
    CommentHeaderTooLarge,
    /// Any String within the comment header claims to be larger than the header itself.
    CommentTooLong,
    /// An error occurred while counting the length of the comment header. This is should not happen and should be considered a bug in this librray.
    LengthMismatch
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
            _ => None
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => e.fmt(f),
            ParseError::Encoding(e) => e.fmt(f),
            ParseError::InvalidOggPage => f.write_str("missing Ogg page magic"),
            ParseError::InvalidOpusHeader => f.write_str("Opus header is missing the magic signature"),
            ParseError::CommentHeaderTooLarge => f.write_str("Opus comment header is larger than 120MB"),
            ParseError::CommentTooLong => f.write_str("A comment claims to be longer than the Header itself"),
            ParseError::LengthMismatch => f.write_str("The length of the comment header does not match the calculated length")
        }
    }
}
