use super::{
    board::Board,
    castling_rights::CastlingRights,
    color::Color,
    double_pawn_push_file::DoublePawnPushFile,
    piece::Piece,
    square::Square,
    zobrist_policy::{ZobristPolicy, private},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WithoutZobrist;

impl private::Sealed for WithoutZobrist {}

impl ZobristPolicy for WithoutZobrist {
    type HashState = ();

    #[inline(always)]
    fn initial_hash(
        _board: &Board,
        _castling_rights: CastlingRights,
        _double_pawn_push_file: DoublePawnPushFile,
        _side_to_move: Color,
    ) -> Self::HashState {
    }

    #[inline(always)]
    fn on_put_piece(_hash: &mut Self::HashState, _piece: Piece, _square: Square) {}

    #[inline(always)]
    fn on_remove_piece(_hash: &mut Self::HashState, _piece: Piece, _square: Square) {}

    #[inline(always)]
    fn on_move_piece(_hash: &mut Self::HashState, _piece: Piece, _from: Square, _to: Square) {}

    #[inline(always)]
    fn on_castling_rights_change(
        _hash: &mut Self::HashState,
        _old: CastlingRights,
        _new: CastlingRights,
    ) {
    }

    #[inline(always)]
    fn on_double_pawn_push_file_change(
        _hash: &mut Self::HashState,
        _old: DoublePawnPushFile,
        _new: DoublePawnPushFile,
    ) {
    }

    #[inline(always)]
    fn on_side_to_move_flip(_hash: &mut Self::HashState) {}

    #[inline(always)]
    fn is_consistent(
        _hash: &Self::HashState,
        _board: &Board,
        _castling_rights: CastlingRights,
        _double_pawn_push_file: DoublePawnPushFile,
        _side_to_move: Color,
    ) -> bool {
        true
    }
}
