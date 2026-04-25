//! Contains [`Position::make_move`].

use crate::Color;
use crate::ColoredPiece;
use crate::Flank;
use crate::Piece;
use crate::Square;
use crate::r#move::{Move, MoveType};
use crate::position::Position;
use crate::position::context::PositionContext;
use crate::{ConstDoublePawnPushFile, DoublePawnPushFile};

impl<const N: usize, const STM: Color> Position<N, STM> {
    fn process_promotion(
        &mut self,
        stm: Color,
        dst_square: Square,
        src_square: Square,
        promotion: Piece,
        new_context: &mut PositionContext,
    ) {
        self.process_possible_capture(stm, dst_square, new_context);

        self.board.remove_piece_at(Piece::Pawn, src_square);
        self.board.put_piece_at(promotion, dst_square);

        new_context.process_promotion_disregarding_capture();
    }

    fn process_normal(
        &mut self,
        stm: Color,
        dst_square: Square,
        src_square: Square,
        new_context: &mut PositionContext,
    ) {
        self.process_possible_capture(stm, dst_square, new_context);

        let moved_piece = self.board.piece_at(src_square);
        debug_assert_ne!(moved_piece, Piece::Null);
        self.board.move_piece(moved_piece, dst_square, src_square);
        new_context.process_normal_disregarding_capture(
            ColoredPiece::new(stm, moved_piece),
            dst_square,
            src_square,
        );
    }

    fn process_possible_capture(
        &mut self,
        stm: Color,
        dst_square: Square,
        new_context: &mut PositionContext,
    ) {
        let opposite_color = stm.other();

        self.board.remove_color_at(opposite_color, dst_square);

        let captured_piece = self.board.piece_at(dst_square);
        if captured_piece != Piece::Null {
            self.board.remove_piece_at(captured_piece, dst_square);
            new_context.process_capture(
                ColoredPiece::new(opposite_color, captured_piece),
                dst_square,
            );
        }
    }

    fn process_en_passant(
        &mut self,
        stm: Color,
        dst_square: Square,
        src_square: Square,
        new_context: &mut PositionContext,
    ) {
        let opposite_color = stm.other();

        let en_passant_capture_square = match opposite_color {
            Color::White => Square::from_u8(dst_square as u8 - 8),
            Color::Black => Square::from_u8(dst_square as u8 + 8),
        };

        self.board
            .remove_color_at(opposite_color, en_passant_capture_square);
        self.board.move_piece(Piece::Pawn, dst_square, src_square);
        self.board
            .remove_piece_at(Piece::Pawn, en_passant_capture_square);

        new_context.process_en_passant();
    }

    fn process_castling(
        &mut self,
        stm: Color,
        dst_square: Square,
        src_square: Square,
        new_context: &mut PositionContext,
    ) {
        self.board.move_piece(Piece::King, dst_square, src_square);

        let (rook_src_square, rook_dst_square) = match dst_square.file().flank() {
            Flank::Kingside => (
                src_square.relative(3).expect("src_square is incorrect"),
                src_square.relative(1).unwrap(),
            ),
            Flank::Queenside => (
                src_square.relative(-4).expect("src_square is incorrect"),
                src_square.relative(-1).unwrap(),
            ),
        };

        self.board.move_colored_piece(
            ColoredPiece::new(stm, Piece::Rook),
            rook_dst_square,
            rook_src_square,
        );

        new_context.process_castling(stm);
    }

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

        self.board.move_color(STM, dst_square, src_square);

        let move_type = mv.move_type();

        match move_type {
            MoveType::Normal | MoveType::DoublePawnPush | MoveType::NormalCapture => {
                self.process_normal(STM, dst_square, src_square, &mut new_context);
            }
            MoveType::Castling => {
                self.process_castling(STM, dst_square, src_square, &mut new_context);
            }
            MoveType::EnPassant => {
                self.process_en_passant(STM, dst_square, src_square, &mut new_context);
            }
            _ => {
                self.process_promotion(
                    STM,
                    dst_square,
                    src_square,
                    move_type.promotion_piece(),
                    &mut new_context,
                );
            }
        }

        self.halfmove += 1;
        self.push_context(new_context);
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
