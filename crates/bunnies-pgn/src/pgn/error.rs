use std::{
    error::Error,
    fmt::{Display, Formatter},
};

/// Represents errors that can occur while lexing or parsing PGN.
#[derive(Debug, PartialEq, Clone)]
pub enum PgnError {
    InvalidMove(String),
    InvalidTag(String),
    InvalidComment(String),
    InvalidMoveNumber(String),
    InvalidCastlingMove(String),
    InvalidToken(String),
    IncorrectMoveNumber(String),
    IllegalMove(String),
    AmbiguousMove(String),
    UnexpectedToken(String),
    UnexpectedEndOfInput(String),
}

impl Display for PgnError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PgnError {}

impl Default for PgnError {
    fn default() -> Self {
        Self::InvalidToken(String::new())
    }
}
