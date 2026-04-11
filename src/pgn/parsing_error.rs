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
    /// Position context stack was full when applying a move: use a larger `N` on [`PgnParser<N>`](crate::pgn::PgnParser) / [`Position<N>`](crate::position::Position) so the longest half-move path fits.
    ContextStackFull,
}

impl Display for PgnParsingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for PgnParsingError {}
