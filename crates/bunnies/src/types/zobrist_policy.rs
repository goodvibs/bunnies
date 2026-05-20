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

pub trait ZobristPolicy:
    private::Sealed + Copy + Clone + Eq + PartialEq + core::fmt::Debug + Default
{
    type HashState: Copy + Clone + Eq + PartialEq + core::fmt::Debug + Default;

    fn initial_hash(
        board: &Board,
        castling_rights: CastlingRights,
        double_pawn_push_file: DoublePawnPushFile,
        side_to_move: Color,
    ) -> Self::HashState;

    fn on_put_piece(hash: &mut Self::HashState, piece: Piece, square: Square);
    fn on_remove_piece(hash: &mut Self::HashState, piece: Piece, square: Square);
    fn on_move_piece(hash: &mut Self::HashState, piece: Piece, from: Square, to: Square);
    fn on_castling_rights_change(
        hash: &mut Self::HashState,
        old: CastlingRights,
        new: CastlingRights,
    );
    fn on_double_pawn_push_file_change(
        hash: &mut Self::HashState,
        old: DoublePawnPushFile,
        new: DoublePawnPushFile,
    );
    fn on_side_to_move_flip(hash: &mut Self::HashState);

    fn is_consistent(
        hash: &Self::HashState,
        board: &Board,
        castling_rights: CastlingRights,
        double_pawn_push_file: DoublePawnPushFile,
        side_to_move: Color,
    ) -> bool;
}
