use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Clone)]
pub enum PgnLexingError {
    InvalidMove(String),
    InvalidTag,
    InvalidComment,
    InvalidVariation,
    InvalidMoveNumber,
    InvalidCastlingMove(String),
    InvalidPromotion,
    InvalidAnnotation,
    InvalidResult,
    InvalidToken
}

impl Display for PgnLexingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Error for PgnLexingError {}

impl Default for PgnLexingError {
    fn default() -> Self {
        PgnLexingError::InvalidToken
    }
}