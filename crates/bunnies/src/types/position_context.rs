//! Context entries pushed for each ply in a [`crate::types::Position`] stack.

use super::{
    bitboard::Bitboard,
    castling_rights::CastlingRights,
    double_pawn_push_file::{ConstDoublePawnPushFile, DoublePawnPushFile},
    piece::Piece,
};

/// A struct containing metadata about the current and past states of the game.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct PositionContext<H = u64> {
    /// Halfmoves since last pawn move or capture (for 50-move rule).
    pub halfmove_clock: u8,
    /// File index for en passant after a double push; see [`DoublePawnPushFile`].
    pub double_pawn_push_file: DoublePawnPushFile,
    /// Castling availability mask for the current position.
    pub castling_rights: CastlingRights,
    /// Captured piece on the move that produced this context, or [`Piece::Null`].
    pub captured_piece: Piece,
    /// Incremental hash state, policy-defined by `H`.
    pub zobrist_hash: H,
    /// Friendly pieces pinned to the king for the side to move.
    pub pinned: Bitboard,
    /// Enemy pieces currently giving check to the side to move.
    pub checkers: Bitboard,
}

impl<H: Default> PositionContext<H> {
    /// Creates a new context with no previous context.
    pub fn blank() -> PositionContext<H> {
        PositionContext {
            halfmove_clock: 0,
            double_pawn_push_file: DoublePawnPushFile::NONE,
            castling_rights: CastlingRights::B0000,
            captured_piece: Piece::Null,
            zobrist_hash: H::default(),
            pinned: 0,
            checkers: 0,
        }
    }

    /// Checks if the halfmove clock is valid (less than or equal to 100).
    pub const fn has_valid_halfmove_clock(&self) -> bool {
        self.halfmove_clock <= 100
    }
}

impl<H: Default> Default for PositionContext<H> {
    fn default() -> Self {
        Self::blank()
    }
}
