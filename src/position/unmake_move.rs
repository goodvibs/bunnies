//! Contains the implementation of the `State::unmake_move` method.

use crate::Color;
use crate::ColoredPiece;
use crate::Flank;
use crate::Piece;
use crate::Square;
use crate::masks::STARTING_KING_ROOK_GAP;
use crate::r#move::{Move, MoveFlag};
use crate::position::{GameResult, Position};

impl<const N: usize> Position<N> {
    fn unprocess_promotion(&mut self, dst_square: Square, src_square: Square, promotion: Piece) {
        self.board.remove_piece_at(promotion, dst_square); // remove promoted piece
        self.board.put_piece_at(Piece::Pawn, src_square); // put pawn back

        self.unprocess_possible_capture(dst_square); // add possible captured piece back
    }

    fn unprocess_normal(&mut self, dst_square: Square, src_square: Square) {
        let moved_piece = self.board.piece_at(dst_square); // get moved piece
        self.board.move_piece(moved_piece, src_square, dst_square); // move piece back

        self.unprocess_possible_capture(dst_square); // add possible captured piece back
    }

    fn unprocess_possible_capture(&mut self, dst_square: Square) {
        // remove captured piece and get captured piece type
        let captured_piece = self.context().captured_piece;
        if captured_piece != Piece::Null {
            // piece was captured
            self.board.put_color_at(self.side_to_move, dst_square); // put captured color back
            self.board.put_piece_at(captured_piece, dst_square); // put captured piece back
        }
    }

    fn unprocess_en_passant(&mut self, dst_square: Square, src_square: Square) {
        let en_passant_capture_square = match self.side_to_move {
            Color::White => unsafe { Square::from(dst_square as u8 - 8) },
            Color::Black => unsafe { Square::from(dst_square as u8 + 8) },
        };

        self.board.move_piece(Piece::Pawn, src_square, dst_square); // move pawn back
        self.board
            .put_color_at(self.side_to_move, en_passant_capture_square); // put captured color back
        self.board
            .put_piece_at(Piece::Pawn, en_passant_capture_square); // put captured piece back
    }

    fn unprocess_castling(&mut self, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.mask();

        self.board.move_piece(Piece::King, src_square, dst_square); // move king back

        let caster = self.side_to_move.other();
        let flank =
            if dst_mask & STARTING_KING_ROOK_GAP[caster as usize][Flank::Kingside as usize] != 0 {
                Flank::Kingside
            } else {
                Flank::Queenside
            };

        let (rook_src_square, rook_dst_square) = match flank {
            Flank::Kingside => (unsafe { Square::from(src_square as u8 + 3) }, unsafe {
                Square::from(src_square as u8 + 1)
            }),
            Flank::Queenside => (unsafe { Square::from(src_square as u8 - 4) }, unsafe {
                Square::from(src_square as u8 - 1)
            }),
        };

        self.board.move_colored_piece(
            ColoredPiece::new(self.side_to_move.other(), Piece::Rook),
            rook_src_square,
            rook_dst_square,
        ); // move rook back
    }

    /// Undoes a move from State without checking if it is valid, legal, or even applied to the current position.
    /// This method is used to undo a move that was previously made with `State::make_move`, regardless of
    /// whether the move was legal. However, the move must have been valid (not malformed).
    pub fn unmake_move(&mut self, mv: Move) {
        let src_square = mv.source();
        let dst_square = mv.destination();

        self.board
            .move_color(self.side_to_move.other(), src_square, dst_square);

        match mv.flag() {
            MoveFlag::NormalMove => self.unprocess_normal(dst_square, src_square),
            MoveFlag::Promotion => self.unprocess_promotion(dst_square, src_square, mv.promotion()),
            MoveFlag::EnPassant => self.unprocess_en_passant(dst_square, src_square),
            MoveFlag::Castling => self.unprocess_castling(dst_square, src_square),
        }

        // update data members
        self.halfmove -= 1;
        self.side_to_move = self.side_to_move.other();
        self.decrement_context_stack_for_unmake();
        self.result = GameResult::None;
    }
}
