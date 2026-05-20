//! PGN parsing and lexing error types.

use std::{
    error::Error,
    fmt::{Display, Formatter},
};

/// Errors that can occur during PGN tokenization or parsing.
#[derive(Debug, PartialEq, Clone)]
pub enum PgnError {
    /// Move syntax didn't match expected SAN format.
    InvalidMove(String),
    /// Tag pair syntax was malformed (e.g., missing brackets).
    InvalidTag(String),
    /// Comment syntax error (unclosed brace).
    InvalidComment(String),
    /// Move number was not a valid number or had wrong suffix.
    InvalidMoveNumber(String),
    /// Castling move didn't match O-O or O-O-O pattern.
    InvalidCastlingMove(String),
    /// Unexpected characters in input stream.
    InvalidToken(String),
    /// Move number out of sequence (e.g., "1. e4 1... e5").
    IncorrectMoveNumber(String),
    /// Move is illegal in the current position.
    IllegalMove(String),
    /// SAN move matches multiple legal moves (insufficient disambiguation).
    AmbiguousMove(String),
    /// Token encountered in wrong context (e.g., tag where move expected).
    UnexpectedToken(String),
    /// Input ended before required token (e.g., missing game result).
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
