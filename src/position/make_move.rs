//! Contains [`Position::make_move`] and [`Position::unmake_move`].

use crate::Color;
use crate::File;
use crate::Flank;
use crate::Piece;
use crate::Rank;
use crate::Square;
use crate::r#move::{Move, MoveFlag};
use crate::position::context::PositionContext;
use crate::position::{GameResult, Position};
use crate::{ConstDoublePawnPushFile, DoublePawnPushFile};

impl<const N: usize, const STM: Color> Position<N, STM> {
    /// Applies a move in place, updating board state for the opponent.
    ///
    /// After this returns, the layout matches the destination type of [`Position::make_move`]
    /// (`Position<N, { STM.other() }>`).
    pub fn make_move(&mut self, mv: Move) {
        debug_assert!(self.context_len() < N);

        let src_square = mv.source();
        let dst_square = mv.destination();
        let flag = mv.flag();

        let mut new_context = PositionContext::blank();
        new_context.halfmove_clock = self.context().halfmove_clock + 1;
        new_context.castling_rights = self.context().castling_rights;

        let piece_at_dst = self.board.piece_at(dst_square);
        if piece_at_dst != Piece::Null {
            self.board
                .remove_piece_and_color(STM.other(), piece_at_dst, dst_square);
            new_context.captured_piece = piece_at_dst;
            new_context.halfmove_clock = 0;
        }

        let piece_at_src = self.board.piece_at(src_square);
        if piece_at_src == Piece::Pawn {
            new_context.halfmove_clock = 0;
            new_context.double_pawn_push_file =
                DoublePawnPushFile::from_pawn_step(dst_square, src_square);
        }

        self.board
            .move_piece_and_color(STM, piece_at_src, dst_square, src_square);

        match flag {
            MoveFlag::Promotion => {
                self.board.remove_piece_at(Piece::Pawn, dst_square);
                self.board.put_piece_at(mv.promotion(), dst_square);
                new_context.halfmove_clock = 0;
            }
            MoveFlag::EnPassant => {
                let capture_square = Square::from_u8(
                    (dst_square as u8).wrapping_add_signed(en_passant_capture_offset(STM)),
                );
                self.board
                    .remove_piece_and_color(STM.other(), Piece::Pawn, capture_square);
                new_context.captured_piece = Piece::Pawn;
                new_context.halfmove_clock = 0;
            }
            MoveFlag::Castling => {
                let flank = dst_square.file().flank();
                let rook_from = castling_rook_from_square(flank, STM);
                let rook_to = castling_rook_to_square(flank, STM);
                self.board.move_color(STM, rook_to, rook_from);
                self.board.move_piece(Piece::Rook, rook_to, rook_from);
                new_context.halfmove_clock = 0;
            }
            _ => {}
        }

        new_context.castling_rights = new_context
            .castling_rights
            .after_move(src_square)
            .after_move(dst_square);

        self.halfmove += 1;
        self.push_context(new_context);
        self.update_pins_and_checks_for_stm(STM.other());
    }

    /// Undoes a move in place. After this, memory matches the destination type of [`Position::unmake_move`].
    pub fn unmake_move(&mut self, mv: Move) {
        let src_square = mv.source();
        let dst_square = mv.destination();
        let flag = mv.flag();
        let mover = STM.other();

        // Color mask up front: undoes the make-side's `move_color` for every move type.
        self.board.move_color(mover, src_square, dst_square);

        match flag {
            MoveFlag::NormalMove => {
                let moved = self.board.piece_at(dst_square);
                self.board.move_piece(moved, src_square, dst_square);
                self.unprocess_possible_capture(dst_square);
            }
            MoveFlag::Promotion => {
                let promoted = self.board.piece_at(dst_square);
                self.board.remove_piece_at(promoted, dst_square);
                self.board.put_piece_at(Piece::Pawn, src_square);
                self.unprocess_possible_capture(dst_square);
            }
            MoveFlag::EnPassant => {
                self.board.move_piece(Piece::Pawn, src_square, dst_square);
                let capture_square = Square::from_u8(
                    (dst_square as u8).wrapping_add_signed(en_passant_capture_offset(mover)),
                );
                self.board.put_color_at(STM, capture_square);
                self.board.put_piece_at(Piece::Pawn, capture_square);
            }
            MoveFlag::Castling => {
                self.board.move_piece(Piece::King, src_square, dst_square);
                let flank = dst_square.file().flank();
                let rook_from = castling_rook_from_square(flank, mover);
                let rook_to = castling_rook_to_square(flank, mover);
                self.board.move_color(mover, rook_from, rook_to);
                self.board.move_piece(Piece::Rook, rook_from, rook_to);
            }
        }

        self.halfmove -= 1;
        // Pins, checkers, castling rights, double-pawn-push file, halfmove clock, and captured
        // piece all live on the previous context; popping restores them verbatim. No recompute.
        self.decrement_context_stack_for_unmake();
        self.result = GameResult::None;
    }

    fn unprocess_possible_capture(&mut self, dst_square: Square) {
        let captured = self.context().captured_piece;
        if captured != Piece::Null {
            self.board.put_color_at(STM, dst_square);
            self.board.put_piece_at(captured, dst_square);
        }
    }
}

const fn en_passant_capture_offset(stm: Color) -> i8 {
    match stm {
        Color::White => 8,
        Color::Black => -8,
    }
}

const fn castling_rook_from_square(flank: Flank, color: Color) -> Square {
    let rank = Rank::One.from_perspective(color);
    match flank {
        Flank::Kingside => Square::from_rank_and_file(rank, File::H),
        Flank::Queenside => Square::from_rank_and_file(rank, File::A),
    }
}

const fn castling_rook_to_square(flank: Flank, color: Color) -> Square {
    let rank = Rank::One.from_perspective(color);
    match flank {
        Flank::Kingside => Square::from_rank_and_file(rank, File::F),
        Flank::Queenside => Square::from_rank_and_file(rank, File::D),
    }
}
