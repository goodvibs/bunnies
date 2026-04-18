//! Context struct and methods

use crate::Bitboard;
use crate::CastlingRights;
use crate::Piece;

use super::DoublePawnPushFile;
use super::DoublePawnPushFileUtils;

/// A struct containing metadata about the current and past states of the game.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct PositionContext {
    pub halfmove_clock: u8,
    /// File index for en passant after a double push; see [`DoublePawnPushFile`].
    pub double_pawn_push_file: DoublePawnPushFile,
    pub castling_rights: CastlingRights,
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
            double_pawn_push_file: DoublePawnPushFile::NONE,
            castling_rights: CastlingRights::NONE,
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
