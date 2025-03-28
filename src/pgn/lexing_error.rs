use std::error::Error;
use std::fmt::{Display, Formatter};

/// Represents errors that can occur during PGN lexing.
#[derive(Debug, PartialEq, Clone)]
pub enum PgnLexingError {
    InvalidMove(String),
    InvalidTag(String),
    InvalidComment(String),
    InvalidMoveNumber(String),
    InvalidCastlingMove(String),
    InvalidToken(String),
}

impl Display for PgnLexingError {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PgnLexingError {}

impl Default for PgnLexingError {
    fn default() -> Self {
        PgnLexingError::InvalidToken("".to_string())
    }
}
