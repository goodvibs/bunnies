//! Contains [`Position::make_move`].

use crate::Bitboard;
use crate::Color;
use crate::ColoredPiece;
use crate::Flank;
use crate::Piece;
use crate::Square;
use crate::masks::{STARTING_KING_ROOK_GAP, STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::r#move::{Move, MoveFlag};
use crate::position::context::PositionContext;
use crate::position::{Position, PositionError};

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
        assert_ne!(moved_piece, Piece::Null);
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
        let dst_mask = dst_square.mask();
        let opposite_color = stm.other();

        self.board.remove_color_at(opposite_color, dst_square);

        let captured_piece = self.board.piece_at(dst_square);
        if captured_piece != Piece::Null {
            self.board.remove_piece_at(captured_piece, dst_square);
            new_context
                .process_capture(ColoredPiece::new(opposite_color, captured_piece), dst_mask);
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
            Color::White => unsafe { Square::from(dst_square as u8 - 8) },
            Color::Black => unsafe { Square::from(dst_square as u8 + 8) },
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
        let dst_mask = dst_square.mask();

        self.board.move_piece(Piece::King, dst_square, src_square);

        let flank =
            if dst_mask & STARTING_KING_ROOK_GAP[stm as usize][Flank::Kingside as usize] != 0 {
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
            ColoredPiece::new(stm, Piece::Rook),
            rook_dst_square,
            rook_src_square,
        );

        new_context.process_castling(stm);
    }

    /// Applies a move in place, updating board state for the opponent.
    /// After this returns `Ok`, the layout matches the destination type of [`Position::make_move`]
    /// (`Position<N, { STM.other() }>`).
    pub(crate) fn make_move_in_place(&mut self, mv: Move) -> Result<(), PositionError> {
        let stm = STM;
        if self.context_len() >= N {
            return Err(PositionError::ContextStackFull);
        }

        let src_square = mv.source();
        let dst_square = mv.destination();

        let mut new_context = PositionContext::blank();
        new_context.halfmove_clock = self.context().halfmove_clock + 1;
        new_context.castling_rights = self.context().castling_rights;

        self.board.move_color(stm, dst_square, src_square);

        match mv.flag() {
            MoveFlag::NormalMove => {
                self.process_normal(stm, dst_square, src_square, &mut new_context);
            }
            MoveFlag::Promotion => {
                self.process_promotion(
                    stm,
                    dst_square,
                    src_square,
                    mv.promotion(),
                    &mut new_context,
                );
            }
            MoveFlag::EnPassant => {
                self.process_en_passant(stm, dst_square, src_square, &mut new_context);
            }
            MoveFlag::Castling => {
                self.process_castling(stm, dst_square, src_square, &mut new_context);
            }
        }

        self.halfmove += 1;
        self.try_push_context(new_context)?;
        self.update_pins_and_checks_for_stm(STM.other());

        Ok(())
    }

    /// Applies a move without checking if it is valid or legal.
    ///
    /// Returns [`PositionError::ContextStackFull`] if the context stack cannot grow (no state change).
    ///
    /// Flipped side is `Position<N, { STM.other() }>`.
    pub fn make_move(
        mut self,
        mv: Move,
    ) -> Result<Position<N, { STM.other() }>, PositionError> {
        self.make_move_in_place(mv)?;
        Ok(self.rebrand_stm::<{ STM.other() }>())
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
        if is_double_pawn_push(dst_square, src_square) {
            self.double_pawn_push = (src_square as u8 % 8) as i8;
        }
    }

    fn process_normal_king_move_disregarding_capture(&mut self, moved_piece_color: Color) {
        self.castling_rights &= !(Flank::Kingside.rights_mask(moved_piece_color)
            | Flank::Queenside.rights_mask(moved_piece_color));
    }

    fn process_normal_rook_move_disregarding_capture(
        &mut self,
        moved_piece_color: Color,
        src_square: Square,
    ) {
        let src_mask = src_square.mask();

        if src_mask & STARTING_KING_SIDE_ROOK[moved_piece_color as usize] != 0 {
            self.castling_rights &= !Flank::Kingside.rights_mask(moved_piece_color);
        } else if src_mask & STARTING_QUEEN_SIDE_ROOK[moved_piece_color as usize] != 0 {
            self.castling_rights &= !Flank::Queenside.rights_mask(moved_piece_color);
        }
    }

    fn process_en_passant(&mut self) {
        self.halfmove_clock = 0;
        self.captured_piece = Piece::Pawn;
    }

    fn process_castling(&mut self, color: Color) {
        self.halfmove_clock = 0;
        self.castling_rights &=
            !(Flank::Kingside.rights_mask(color) | Flank::Queenside.rights_mask(color));
    }

    fn process_capture(&mut self, captured_colored_piece: ColoredPiece, dst_mask: Bitboard) {
        let captured_color = captured_colored_piece.color();
        let captured_piece = captured_colored_piece.piece();

        self.captured_piece = captured_piece;
        self.halfmove_clock = 0;
        if captured_piece == Piece::Rook {
            let king_side_rook_mask = STARTING_KING_SIDE_ROOK[captured_color as usize];
            let queen_side_rook_mask = STARTING_QUEEN_SIDE_ROOK[captured_color as usize];
            if dst_mask & king_side_rook_mask != 0 {
                self.castling_rights &= !Flank::Kingside.rights_mask(captured_color);
            } else if dst_mask & queen_side_rook_mask != 0 {
                self.castling_rights &= !Flank::Queenside.rights_mask(captured_color);
            }
        }
    }
}

const fn is_double_pawn_push(dst_square: Square, src_square: Square) -> bool {
    let dst_mask = dst_square.mask();
    let src_mask = src_square.mask();

    dst_mask & (src_mask << 16) != 0 || dst_mask & (src_mask >> 16) != 0
}
