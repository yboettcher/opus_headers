use std::io;
use std::str;

#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    Encoding(str::Utf8Error),
    InvalidOggPage,
    InvalidOpusHeader,
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
