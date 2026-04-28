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
        let captured_piece = self.context().captured_piece;

        match move_type {
            MoveType::Normal | MoveType::DoublePawnPush => {
                let moved_piece = self.board.piece_at(dst_square);
                debug_assert_ne!(moved_piece, Piece::Null);
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    moved_piece,
                    captured_piece,
                    move_type,
                );
            }
            MoveType::NormalCapture => {
                let moved_piece = self.board.piece_at(dst_square);
                debug_assert_ne!(moved_piece, Piece::Null);
                debug_assert_ne!(captured_piece, Piece::Null);
                debug_assert_ne!(captured_piece, Piece::King);
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    moved_piece,
                    captured_piece,
                    move_type,
                );
            }
            MoveType::Castling => {
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    Piece::King,
                    captured_piece,
                    move_type,
                );
            }
            MoveType::EnPassant => {
                debug_assert_eq!(captured_piece, Piece::Pawn);
                self.board.apply_move::<false>(
                    dst_square,
                    src_square,
                    mover,
                    Piece::Pawn,
                    Piece::Null,
                    move_type,
                );
            }
            MoveType::PushPromotionToKnight
            | MoveType::PushPromotionToBishop
            | MoveType::PushPromotionToRook
            | MoveType::PushPromotionToQueen
            | MoveType::CapturePromotionToKnight
            | MoveType::CapturePromotionToBishop
            | MoveType::CapturePromotionToRook
            | MoveType::CapturePromotionToQueen => {
                let promotion_piece = move_type.promotion_piece();
                debug_assert_ne!(promotion_piece, Piece::Null);
                if move_type.is_capture() {
                    debug_assert_ne!(captured_piece, Piece::Null);
                    debug_assert_ne!(captured_piece, Piece::King);
                    self.board.apply_move::<false>(
                        dst_square,
                        src_square,
                        mover,
                        Piece::Pawn,
                        captured_piece,
                        move_type,
                    );
                } else {
                    debug_assert_eq!(captured_piece, Piece::Null);
                    self.board.apply_move::<false>(
                        dst_square,
                        src_square,
                        mover,
                        Piece::Pawn,
                        captured_piece,
                        move_type,
                    );
                }
            }
        }

        self.halfmove -= 1;
        self.decrement_context_stack_for_unmake();
        self.result = GameResult::None;
        self.update_pins_and_checks_for_stm(STM.other());
    }
}
