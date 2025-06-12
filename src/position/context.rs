//! Context struct and methods

use crate::Bitboard;
use crate::Piece;

/// A struct containing metadata about the current and past states of the game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct PositionContext {
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8,  // 0, 0, 0, 0, wk, wq, bk, bq

    // updated after every move
    pub captured_piece: Piece,
    pub zobrist_hash: Bitboard,
    pub pinned: Bitboard,
    pub checkers: Bitboard,
}

impl PositionContext {
    /// Creates a new context with no previous context.
    pub const fn blank() -> PositionContext {
        PositionContext {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0,
            captured_piece: Piece::Null,
            zobrist_hash: 0,
            pinned: 0,
            checkers: 0,
        }
    }

    /// Checks if the halfmove clock is valid (less than or equal to 100).
    pub const fn has_valid_halfmove_clock(&self) -> bool {
        self.halfmove_clock <= 100
    }
}

impl Default for PositionContext {
    fn default() -> Self {
        Self::blank()
    }
}
