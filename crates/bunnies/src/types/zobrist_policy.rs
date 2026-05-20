//! Abstraction over incremental hash maintenance for [`crate::types::Position`].
//!
//! Implementations can either keep a real Zobrist hash (`WithZobrist`) or a no-op
//! placeholder state (`WithoutZobrist`) for builds that do not need hashing.

use super::{
    board::Board,
    castling_rights::CastlingRights,
    color::Color,
    double_pawn_push_file::DoublePawnPushFile,
    piece::Piece,
    square::Square,
};

pub(crate) mod private {
    pub trait Sealed {}
}

/// Policy trait used by [`crate::types::Position`] to maintain per-context hash state.
pub trait ZobristPolicy:
    private::Sealed + Copy + Clone + Eq + PartialEq + core::fmt::Debug + Default
{
    /// Storage type for the hash payload (`u64` for real Zobrist, `()` for no-op).
    type HashState: Copy + Clone + Eq + PartialEq + core::fmt::Debug + Default;

    /// Computes the initial hash state for a freshly created position context.
    fn initial_hash(
        board: &Board,
        castling_rights: CastlingRights,
        double_pawn_push_file: DoublePawnPushFile,
        side_to_move: Color,
    ) -> Self::HashState;

    /// Applies an incremental hash update for placing `piece` on `square`.
    fn on_put_piece(hash: &mut Self::HashState, piece: Piece, square: Square);
    /// Applies an incremental hash update for removing `piece` from `square`.
    fn on_remove_piece(hash: &mut Self::HashState, piece: Piece, square: Square);
    /// Applies an incremental hash update for moving `piece` from `from` to `to`.
    fn on_move_piece(hash: &mut Self::HashState, piece: Piece, from: Square, to: Square);
    /// Applies an incremental hash update for castling-rights change.
    fn on_castling_rights_change(
        hash: &mut Self::HashState,
        old: CastlingRights,
        new: CastlingRights,
    );
    /// Applies an incremental hash update for en-passant-file change.
    fn on_double_pawn_push_file_change(
        hash: &mut Self::HashState,
        old: DoublePawnPushFile,
        new: DoublePawnPushFile,
    );
    /// Applies side-to-move toggle to the hash state.
    fn on_side_to_move_flip(hash: &mut Self::HashState);

    /// Returns whether stored `hash` matches the recomputed canonical hash for current state.
    fn is_consistent(
        hash: &Self::HashState,
        board: &Board,
        castling_rights: CastlingRights,
        double_pawn_push_file: DoublePawnPushFile,
        side_to_move: Color,
    ) -> bool;
}
