//! Contains [`Position::unmake_move`].

use crate::Color;
use crate::Piece;
use crate::r#move::{Move, MoveType};
use crate::position::{GameResult, Position};

impl<const N: usize, const STM: Color> Position<N, STM> {
    /// Undoes a move in place. After this, memory matches the destination type of [`Position::unmake_move`].
    pub fn unmake_move(&mut self, mv: Move) {
        let src_square = mv.source();
        let dst_square = mv.destination();
        let move_type = mv.move_type();
        let mover = STM.other();

        match move_type {
            MoveType::Castling => {
                debug_assert_eq!(self.context().captured_piece, Piece::Null);
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    Piece::King,
                    Piece::Null,
                    move_type,
                );
            }
            MoveType::EnPassant => {
                debug_assert_eq!(self.context().captured_piece, Piece::Pawn);
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    Piece::Pawn,
                    Piece::Null,
                    move_type,
                );
            }
            MoveType::Normal | MoveType::NormalCapture => {
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    self.board.piece_at(dst_square),
                    self.context().captured_piece,
                    move_type,
                );
            }
            _ => {
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    Piece::Pawn,
                    self.context().captured_piece,
                    move_type,
                );
            }
        }

        self.halfmove -= 1;
        self.decrement_context_stack_for_unmake();
        self.result = GameResult::None;
        self.update_pins_and_checks_for_stm(STM.other());
    }
}
