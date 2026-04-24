//! Contains [`Position::make_move`] and [`Position::unmake_move`].

use std::hint;

use crate::Color;
use crate::ColoredPiece;
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
        let moved = self.board.piece_at(src_square);

        let mut new_context = PositionContext::blank();
        new_context.halfmove_clock = self.context().halfmove_clock + 1;
        new_context.castling_rights = self.context().castling_rights;

        // Capture detection lives here only for NormalMove and Promotion; EnPassant handles its
        // own capture inside the match, and Castling is never a capture.
        if matches!(flag, MoveFlag::NormalMove | MoveFlag::Promotion) {
            let captured = self.board.piece_at(dst_square);
            if captured != Piece::Null {
                self.board
                    .remove_colored_piece_at(ColoredPiece::new(STM.other(), captured), dst_square);
                new_context.captured_piece = captured;
                new_context.halfmove_clock = 0;
            }
        }

        match flag {
            MoveFlag::Castling => {
                self.board.move_colored_piece(
                    ColoredPiece::new(STM, Piece::King),
                    dst_square,
                    src_square,
                );
                let flank = dst_square.file().flank();
                let rook_from = castling_rook_from_square(flank, STM);
                let rook_to = castling_rook_to_square(flank, STM);
                self.board.move_colored_piece(
                    ColoredPiece::new(STM, Piece::Rook),
                    rook_to,
                    rook_from,
                );
                new_context.halfmove_clock = 0;
            }
            MoveFlag::EnPassant => {
                self.board.move_colored_piece(
                    ColoredPiece::new(STM, Piece::Pawn),
                    dst_square,
                    src_square,
                );
                let capture_square =
                    Square::from_rank_and_file(STM.en_passant_capture_rank(), dst_square.file());
                self.board.remove_colored_piece_at(
                    ColoredPiece::new(STM.other(), Piece::Pawn),
                    capture_square,
                );
                new_context.captured_piece = Piece::Pawn;
                new_context.halfmove_clock = 0;
            }
            MoveFlag::Promotion => {
                self.board
                    .remove_colored_piece_at(ColoredPiece::new(STM, Piece::Pawn), src_square);
                self.board
                    .put_colored_piece_at(ColoredPiece::new(STM, mv.promotion()), dst_square);
                new_context.halfmove_clock = 0;
            }
            MoveFlag::NormalMove => {
                self.board
                    .remove_colored_piece_at(ColoredPiece::new(STM, moved), src_square);
                self.board
                    .put_colored_piece_at(ColoredPiece::new(STM, moved), dst_square);
                if moved == Piece::Pawn {
                    new_context.halfmove_clock = 0;
                    // Returns NONE for non-double-push pawn steps, so it is safe to call unconditionally.
                    new_context.double_pawn_push_file =
                        DoublePawnPushFile::from_pawn_step(dst_square, src_square);
                }
            }
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
        let moved = self.board.piece_at(dst_square);

        let src_piece = if hint::unlikely(flag == MoveFlag::Promotion) {
            Piece::Pawn
        } else {
            moved
        };

        self.board
            .remove_colored_piece_at(ColoredPiece::new(mover, moved), dst_square);

        self.board
            .put_colored_piece_at(ColoredPiece::new(mover, src_piece), src_square);

        match flag {
            MoveFlag::Castling => {
                let flank = dst_square.file().flank();
                let rook_from = castling_rook_from_square(flank, mover);
                let rook_to = castling_rook_to_square(flank, mover);

                self.board.move_colored_piece(
                    ColoredPiece::new(mover, Piece::Rook),
                    rook_from,
                    rook_to,
                );
            }
            MoveFlag::EnPassant => {
                let capture_square =
                    Square::from_rank_and_file(mover.en_passant_capture_rank(), dst_square.file());

                self.board
                    .put_colored_piece_at(ColoredPiece::new(STM, Piece::Pawn), capture_square);
            }
            MoveFlag::NormalMove | MoveFlag::Promotion => {
                let captured = self.context().captured_piece;
                if captured != Piece::Null {
                    self.board
                        .put_colored_piece_at(ColoredPiece::new(STM, captured), dst_square);
                }
            }
        }

        self.halfmove -= 1;
        // Popping is enough to restore castling rights, the double-pawn-push file, the halfmove
        // clock, and the captured piece -- they all live on the previous context. Pins and checkers
        // are recomputed below since they describe the side now to move.
        self.decrement_context_stack_for_unmake();
        self.result = GameResult::None;
        self.update_pins_and_checks_for_stm(STM.other());
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
