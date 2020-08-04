use std::fmt;
use std::error::Error;


/// An error to indicate that a reader did not read the amount of bytes that was requested.
#[derive(Debug)]
pub struct DidNotReadEnough {
    pub expected: usize,
    pub got: usize
}
impl Error for DidNotReadEnough {}
impl fmt::Display for DidNotReadEnough {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("Only read {} bytes, expected {}", self.got, self.expected))
    }
}

/// An Error to indicate that we were unable to find both headers.
#[derive(Debug)]
pub struct DidNotFindBothHeaders;
impl Error for DidNotFindBothHeaders {}
impl fmt::Display for DidNotFindBothHeaders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Could not find either the identification or comment header in the given Reader")
    }
} 
