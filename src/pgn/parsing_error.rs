use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
/// Represents errors that can occur during PGN parsing.
pub enum PgnParsingError {
    InvalidTag(String),
    IncorrectMoveNumber(String),
    IllegalMove(String),
    AmbiguousMove(String),
    UnexpectedToken(String),
    UnexpectedEndOfInput(String),
    LexingError(String),
}

impl Display for PgnParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PgnParsingError {}
