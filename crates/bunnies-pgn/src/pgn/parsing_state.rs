//! Parser state machine states for PGN token processing.

/// Current section of the PGN file being parsed.
#[derive(Debug)]
#[derive_const(PartialEq)]
pub enum PgnParsingState {
    /// Parsing tag pairs at the beginning (e.g., `[Event "..."]`).
    Tags,
    /// Parsing move text and variations.
    Moves {
        /// Whether the previous token was a move number (to detect "1. e4 e5 2." format).
        move_number_just_seen: bool,
    },
    /// Game result token (1-0, 0-1, 1/2-1/2, *) has been found.
    ResultFound,
}
