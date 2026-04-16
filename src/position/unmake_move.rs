//! Contains [`Position::unmake_move`].

use crate::Color;
use crate::ColoredPiece;
use crate::Flank;
use crate::Piece;
use crate::Square;
use crate::masks::STARTING_KING_ROOK_GAP;
use crate::r#move::{Move, MoveFlag};
use crate::position::{GameResult, Position};

impl<const N: usize, const STM: Color> Position<N, STM> {
    fn unprocess_promotion(&mut self, dst_square: Square, src_square: Square, promotion: Piece) {
        self.board.remove_piece_at(promotion, dst_square);
        self.board.put_piece_at(Piece::Pawn, src_square);

        self.unprocess_possible_capture(dst_square);
    }

    fn unprocess_normal(&mut self, dst_square: Square, src_square: Square) {
        let moved_piece = self.board.piece_at(dst_square);
        self.board.move_piece(moved_piece, src_square, dst_square);

        self.unprocess_possible_capture(dst_square);
    }

    fn unprocess_possible_capture(&mut self, dst_square: Square) {
        let captured_piece = self.context().captured_piece;
        if captured_piece != Piece::Null {
            self.board.put_color_at(STM, dst_square);
            self.board.put_piece_at(captured_piece, dst_square);
        }
    }

    fn unprocess_en_passant(&mut self, dst_square: Square, src_square: Square) {
        let en_passant_capture_square = match STM {
            Color::White => unsafe { Square::from(dst_square as u8 - 8) },
            Color::Black => unsafe { Square::from(dst_square as u8 + 8) },
        };

        self.board.move_piece(Piece::Pawn, src_square, dst_square);
        self.board
            .put_color_at(STM, en_passant_capture_square);
        self.board
            .put_piece_at(Piece::Pawn, en_passant_capture_square);
    }

    fn unprocess_castling(&mut self, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.mask();

        self.board.move_piece(Piece::King, src_square, dst_square);

        let caster = STM.other();
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
            ColoredPiece::new(STM.other(), Piece::Rook),
            rook_src_square,
            rook_dst_square,
        );
    }

    /// Undoes a move in place. After this, memory matches the destination type of [`Position::unmake_move`].
    pub(crate) fn unmake_move_in_place(&mut self, mv: Move) {
        let src_square = mv.source();
        let dst_square = mv.destination();

        self.board
            .move_color(STM.other(), src_square, dst_square);

        match mv.flag() {
            MoveFlag::NormalMove => self.unprocess_normal(dst_square, src_square),
            MoveFlag::Promotion => self.unprocess_promotion(dst_square, src_square, mv.promotion()),
            MoveFlag::EnPassant => self.unprocess_en_passant(dst_square, src_square),
            MoveFlag::Castling => self.unprocess_castling(dst_square, src_square),
        }

        self.halfmove -= 1;
        self.decrement_context_stack_for_unmake();
        self.result = GameResult::None;
    }

}

impl<const N: usize> Position<N, { Color::White }> {
    /// Undoes a move from State without checking if it is valid, legal, or even applied to the current position.
    pub fn unmake_move(mut self, mv: Move) -> Position<N, { Color::Black }> {
        self.unmake_move_in_place(mv);
        let Position {
            board,
            halfmove,
            result,
            contexts,
            context_len,
        } = self;
        Position::<N, { Color::Black }> {
            board,
            halfmove,
            result,
            contexts,
            context_len,
        }
    }
}

impl<const N: usize> Position<N, { Color::Black }> {
    /// Undoes a move from State without checking if it is valid, legal, or even applied to the current position.
    pub fn unmake_move(mut self, mv: Move) -> Position<N, { Color::White }> {
        self.unmake_move_in_place(mv);
        let Position {
            board,
            halfmove,
            result,
            contexts,
            context_len,
        } = self;
        Position::<N, { Color::White }> {
            board,
            halfmove,
            result,
            contexts,
            context_len,
        }
    }
}
