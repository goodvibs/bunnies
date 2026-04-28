//! Contains [`Position::make_move`] and [`Position::unmake_move`].

use crate::Color;
use crate::ColoredPiece;
use crate::Flank;
use crate::Piece;
use crate::Square;
use crate::r#move::{Move, MoveType};
use crate::position::{GameResult, Position};
use crate::position::context::PositionContext;
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

        let mut new_context = PositionContext::blank();
        new_context.halfmove_clock = self.context().halfmove_clock + 1;
        new_context.castling_rights = self.context().castling_rights;

        let move_type = mv.move_type();
        let moved_piece = self.board.piece_at(src_square);
        let captured_piece = self.board.piece_at(dst_square);

        match move_type {
            MoveType::Castling => {
                self.board.apply_move::<true>(
                    dst_square,
                    src_square,
                    STM,
                    Piece::King,
                    Piece::Null,
                    move_type,
                );

                new_context.process_castling(STM);
            }
            MoveType::EnPassant => {
                self.board.apply_move::<true>(
                    dst_square,
                    src_square,
                    STM,
                    Piece::Pawn,
                    Piece::Null,
                    move_type,
                );

                new_context.process_en_passant();
            }
            MoveType::Normal | MoveType::DoublePawnPush | MoveType::NormalCapture => {
                self.board.apply_move::<true>(
                    dst_square,
                    src_square,
                    STM,
                    moved_piece,
                    captured_piece,
                    move_type,
                );

                new_context.process_normal_disregarding_capture(
                    ColoredPiece::new(STM, moved_piece),
                    dst_square,
                    src_square,
                );
            }
            _ => {
                self.board.apply_move::<true>(
                    dst_square,
                    src_square,
                    STM,
                    Piece::Pawn,
                    captured_piece,
                    move_type,
                );

                new_context.process_promotion_disregarding_capture();
            }
        }

        if move_type.is_capture() && move_type != MoveType::EnPassant {
            new_context.process_capture(ColoredPiece::new(STM.other(), captured_piece), dst_square);
        }

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

impl PositionContext {
    fn process_promotion_disregarding_capture(&mut self) {
        self.halfmove_clock = 0;
    }

    fn process_normal_disregarding_capture(
        &mut self,
        moved_piece: ColoredPiece,
        dst_square: Square,
        src_square: Square,
    ) {
        let moved_piece_type = moved_piece.piece();
        let moved_piece_color = moved_piece.color();

        match moved_piece_type {
            Piece::Pawn => {
                self.process_normal_pawn_move_disregarding_capture(dst_square, src_square)
            }
            Piece::King => self.process_normal_king_move_disregarding_capture(moved_piece_color),
            Piece::Rook => {
                self.process_normal_rook_move_disregarding_capture(moved_piece_color, src_square)
            }
            _ => {}
        }
    }

    fn process_normal_pawn_move_disregarding_capture(
        &mut self,
        dst_square: Square,
        src_square: Square,
    ) {
        self.halfmove_clock = 0;
        self.double_pawn_push_file = DoublePawnPushFile::from_pawn_step(dst_square, src_square);
    }

    fn process_normal_king_move_disregarding_capture(&mut self, moved_piece_color: Color) {
        self.castling_rights = self.castling_rights.with_cleared_color(moved_piece_color);
    }

    fn process_normal_rook_move_disregarding_capture(
        &mut self,
        moved_piece_color: Color,
        src_square: Square,
    ) {
        self.castling_rights = match (moved_piece_color, src_square) {
            (Color::White, Square::H1) | (Color::Black, Square::H8) => self
                .castling_rights
                .with_cleared(Flank::Kingside, moved_piece_color),
            (Color::White, Square::A1) | (Color::Black, Square::A8) => self
                .castling_rights
                .with_cleared(Flank::Queenside, moved_piece_color),
            _ => self.castling_rights,
        };
    }

    fn process_en_passant(&mut self) {
        self.halfmove_clock = 0;
        self.captured_piece = Piece::Pawn;
    }

    fn process_castling(&mut self, color: Color) {
        self.halfmove_clock = 0;
        self.castling_rights = self.castling_rights.with_cleared_color(color);
    }

    fn process_capture(&mut self, captured_colored_piece: ColoredPiece, dst_square: Square) {
        let captured_color = captured_colored_piece.color();
        let captured_piece = captured_colored_piece.piece();

        self.captured_piece = captured_piece;
        self.halfmove_clock = 0;
        if captured_piece == Piece::Rook {
            self.castling_rights = match (captured_color, dst_square) {
                (Color::White, Square::H1) | (Color::Black, Square::H8) => self
                    .castling_rights
                    .with_cleared(Flank::Kingside, captured_color),
                (Color::White, Square::A1) | (Color::Black, Square::A8) => self
                    .castling_rights
                    .with_cleared(Flank::Queenside, captured_color),
                _ => self.castling_rights,
            };
        }
    }
}
