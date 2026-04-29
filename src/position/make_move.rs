//! Contains [`Position::make_move`] and [`Position::unmake_move`].

use std::hint;

use crate::Color;
use crate::ColoredPiece;
use crate::File;
use crate::Flank;
use crate::Piece;
use crate::Rank;
use crate::Square;
use crate::r#move::{Move, MoveType};
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
        let move_type = mv.move_type();
        let moved = self.board.piece_at(src_square);
        let captured = self.board.piece_at(dst_square);

        let mut new_context = PositionContext::blank();
        new_context.halfmove_clock = self.context().halfmove_clock + 1;
        new_context.castling_rights = self.context().castling_rights;

        if move_type.is_capture() && move_type != MoveType::EnPassant {
            self.board
                .remove_colored_piece_at(ColoredPiece::new(STM.other(), captured), dst_square);
            new_context.captured_piece = captured;
            new_context.halfmove_clock = 0;
        }

        let dst_piece = if hint::unlikely(move_type.is_promotion()) {
            move_type.promotion_piece()
        } else {
            moved
        };

        match move_type {
            MoveType::Castling => {
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
            MoveType::EnPassant => {
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
            MoveType::DoublePawnPush => {
                self.board.move_colored_piece(
                    ColoredPiece::new(STM, Piece::Pawn),
                    dst_square,
                    src_square,
                );
                new_context.double_pawn_push_file =
                    DoublePawnPushFile::from_pawn_step(dst_square, src_square);
                new_context.halfmove_clock = 0;
            }
            _ => {
                self.board
                    .remove_colored_piece_at(ColoredPiece::new(STM, moved), src_square);
                self.board
                    .put_colored_piece_at(ColoredPiece::new(STM, dst_piece), dst_square);
                if moved == Piece::Pawn {
                    new_context.halfmove_clock = 0;
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
        let move_type = mv.move_type();
        let mover = STM.other();
        let moved = self.board.piece_at(dst_square);

        let src_piece = if hint::unlikely(move_type.is_promotion()) {
            Piece::Pawn
        } else {
            moved
        };

        self.board
            .remove_colored_piece_at(ColoredPiece::new(mover, moved), dst_square);

        self.board
            .put_colored_piece_at(ColoredPiece::new(mover, src_piece), src_square);

        match move_type {
            MoveType::Castling => {
                let flank = dst_square.file().flank();
                let rook_from = castling_rook_from_square(flank, mover);
                let rook_to = castling_rook_to_square(flank, mover);

                self.board.move_colored_piece(
                    ColoredPiece::new(mover, Piece::Rook),
                    rook_from,
                    rook_to,
                );
            }
            MoveType::EnPassant => {
                let capture_square =
                    Square::from_rank_and_file(mover.en_passant_capture_rank(), dst_square.file());

                self.board
                    .put_colored_piece_at(ColoredPiece::new(STM, Piece::Pawn), capture_square);
            }
            _ => {}
        }

        if move_type.is_capture() && move_type != MoveType::EnPassant {
            self.board.put_colored_piece_at(
                ColoredPiece::new(STM, self.context().captured_piece),
                dst_square,
            );
        }

        self.halfmove -= 1;
        self.decrement_context_stack_for_unmake();
        self.result = GameResult::None;
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
