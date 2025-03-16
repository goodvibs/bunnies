use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum PgnParseError {
    InvalidTag(String),
    IncorrectMoveNumber(String),
    IllegalMove(String),
    AmbiguousMove(String),
    UnexpectedToken(String),
}

impl Display for PgnParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PgnParseError {}