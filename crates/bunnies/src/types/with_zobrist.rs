use super::board::Board;
use super::castling_rights::CastlingRights;
use super::color::Color;
use super::double_pawn_push_file::DoublePawnPushFile;
use super::piece::Piece;
use super::square::Square;
use super::zobrist_policy::{ZobristPolicy, private};
use crate::logic::zobrist_hash::{
    calc_position_zobrist_hash, castling_rights_key, double_pawn_push_key, piece_square_key,
    side_to_move_key,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WithZobrist;

impl private::Sealed for WithZobrist {}

impl ZobristPolicy for WithZobrist {
    type HashState = u64;

    #[inline(always)]
    fn initial_hash(
        board: &Board,
        castling_rights: CastlingRights,
        double_pawn_push_file: DoublePawnPushFile,
        side_to_move: Color,
    ) -> Self::HashState {
        calc_position_zobrist_hash(board, castling_rights, double_pawn_push_file, side_to_move)
    }

    #[inline(always)]
    fn on_put_piece(hash: &mut Self::HashState, piece: Piece, square: Square) {
        *hash ^= piece_square_key(Piece::Null, square) ^ piece_square_key(piece, square);
    }

    #[inline(always)]
    fn on_remove_piece(hash: &mut Self::HashState, piece: Piece, square: Square) {
        *hash ^= piece_square_key(piece, square) ^ piece_square_key(Piece::Null, square);
    }

    #[inline(always)]
    fn on_move_piece(hash: &mut Self::HashState, piece: Piece, from: Square, to: Square) {
        *hash ^= piece_square_key(piece, from)
            ^ piece_square_key(Piece::Null, from)
            ^ piece_square_key(Piece::Null, to)
            ^ piece_square_key(piece, to);
    }

    #[inline(always)]
    fn on_castling_rights_change(
        hash: &mut Self::HashState,
        old: CastlingRights,
        new: CastlingRights,
    ) {
        *hash ^= castling_rights_key(old) ^ castling_rights_key(new);
    }

    #[inline(always)]
    fn on_double_pawn_push_file_change(
        hash: &mut Self::HashState,
        old: DoublePawnPushFile,
        new: DoublePawnPushFile,
    ) {
        *hash ^= double_pawn_push_key(old) ^ double_pawn_push_key(new);
    }

    #[inline(always)]
    fn on_side_to_move_flip(hash: &mut Self::HashState) {
        *hash ^= side_to_move_key(Color::Black);
    }

    #[inline(always)]
    fn is_consistent(
        hash: &Self::HashState,
        board: &Board,
        castling_rights: CastlingRights,
        double_pawn_push_file: DoublePawnPushFile,
        side_to_move: Color,
    ) -> bool {
        *hash
            == calc_position_zobrist_hash(
                board,
                castling_rights,
                double_pawn_push_file,
                side_to_move,
            )
    }
}
