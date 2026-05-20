//! Contains [`Position::make_move`] and [`Position::unmake_move`].

use crate::types::{
    Color,
    ConstDoublePawnPushFile,
    DoublePawnPushFile,
    File,
    Flank,
    Move,
    MoveFlag,
    Piece,
    Position,
    PositionContext,
    Rank,
    Square,
    ZobristPolicy,
};

impl<const N: usize, const STM: Color, Z: ZobristPolicy> Position<N, STM, Z> {
    /// Applies a move in place, updating board state for the opponent.
    ///
    /// After this returns, the layout matches the resulting type of [`Position::make_move`]
    /// (`Position<N, { STM.other() }>`).
    pub fn make_move(&mut self, move_: Move) {
        debug_assert!(self.num_contexts < N);

        let from = move_.from();
        let to = move_.to();
        let flag = move_.flag();

        let old_context = *self.context();
        let mut new_context = PositionContext::<Z::HashState>::blank();
        new_context.halfmove_clock = old_context.halfmove_clock + 1;
        new_context.castling_rights = old_context.castling_rights;
        new_context.double_pawn_push_file = old_context.double_pawn_push_file;
        new_context.zobrist_hash = old_context.zobrist_hash;
        self.push_context(new_context);

        let piece_at_to = self.board.piece_at(to);
        if piece_at_to != Piece::Null {
            self.remove_piece_and_color(STM.other(), piece_at_to, to);
            let context = self.mut_context();
            context.captured_piece = piece_at_to;
            context.halfmove_clock = 0;
        }

        let piece_at_from = self.board.piece_at(from);
        if piece_at_from == Piece::Pawn {
            self.mut_context().halfmove_clock = 0;
            self.set_double_pawn_push_file(DoublePawnPushFile::from_pawn_step(from, to));
        } else {
            self.set_double_pawn_push_file(DoublePawnPushFile::NONE);
        }

        self.move_piece_and_color(STM, piece_at_from, from, to);

        match flag {
            MoveFlag::Promotion => {
                self.remove_piece_at(Piece::Pawn, to);
                self.put_piece_at(move_.promotion(), to);
                self.mut_context().halfmove_clock = 0;
            }
            MoveFlag::EnPassant => {
                let capture_square = unsafe {
                    Square::try_from((to as u8).wrapping_add_signed(en_passant_capture_offset(STM)))
                        .unwrap_unchecked()
                };
                self.remove_piece_and_color(STM.other(), Piece::Pawn, capture_square);
                let context = self.mut_context();
                context.captured_piece = Piece::Pawn;
                context.halfmove_clock = 0;
            }
            MoveFlag::Castling => {
                let flank = to.file().flank();
                let rook_from = castling_rook_from_square(flank, STM);
                let rook_to = castling_rook_to_square(flank, STM);
                self.move_color(STM, rook_from, rook_to);
                self.move_piece(Piece::Rook, rook_from, rook_to);
                self.mut_context().halfmove_clock = 0;
            }
            _ => {}
        }

        let castling_rights = self
            .context()
            .castling_rights
            .after_move(from)
            .after_move(to);
        self.set_castling_rights(castling_rights);
        self.flip_side_to_move_hash();

        self.halfmove += 1;
        self.update_pins_and_checks_for_stm(STM.other());
    }

    /// Undoes a move in place. After this, memory matches the resulting type of [`Position::unmake_move`].
    pub fn unmake_move(&mut self, move_: Move) {
        let from = move_.from();
        let to = move_.to();
        let flag = move_.flag();
        let side_just_moved = STM.other();

        let piece_at_to = self.board.piece_at(to);
        self.move_piece_and_color(side_just_moved, piece_at_to, to, from);

        let captured_piece = self.context().captured_piece;
        if captured_piece != Piece::Null {
            self.put_piece_and_color(STM, captured_piece, to);
        }

        match flag {
            MoveFlag::NormalMove => {}
            MoveFlag::Promotion => {
                let promoted = self.board.piece_at(from);
                self.remove_piece_at(promoted, from);
                self.put_piece_at(Piece::Pawn, from);
            }
            MoveFlag::EnPassant => {
                let capture_square = unsafe {
                    Square::try_from(
                        (to as u8).wrapping_add_signed(en_passant_capture_offset(side_just_moved)),
                    )
                    .unwrap_unchecked()
                };
                self.move_piece_and_color(STM, Piece::Pawn, to, capture_square);
            }
            MoveFlag::Castling => {
                let flank = to.file().flank();
                let rook_from = castling_rook_from_square(flank, side_just_moved);
                let rook_to = castling_rook_to_square(flank, side_just_moved);
                self.move_piece_and_color(side_just_moved, Piece::Rook, rook_to, rook_from);
            }
        }

        self.flip_side_to_move_hash();

        self.halfmove -= 1;
        self.decrement_context_stack_for_unmake();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MoveList, PositionWithZobrist, PositionWithoutZobrist};

    fn assert_hash_consistency_after_plies<const N: usize, const STM: Color>(
        pos: &mut PositionWithZobrist<N, STM>,
        plies: u8,
    ) {
        if !pos.is_zobrist_consistent() {
            let ctx = pos.context();
            let expected = crate::logic::zobrist_hash::calc_position_zobrist_hash(
                &pos.board,
                ctx.castling_rights,
                ctx.double_pawn_push_file,
                STM,
            );
            panic!(
                "zobrist mismatch: stm={STM:?} plies={plies} got={:#018x} expected={:#018x}",
                ctx.zobrist_hash, expected
            );
        }
        if plies == 0 {
            return;
        }

        let mut moves = MoveList::new();
        pos.generate_moves(&mut moves);
        let mv = *moves.as_slice().first().expect("at least one legal move");
        pos.make_move(mv);

        match STM {
            Color::White => {
                let child = unsafe { pos.rebrand_stm_mut::<{ Color::Black }>() };
                assert_hash_consistency_after_plies(child, plies - 1);
                child.unmake_move(mv);
            }
            Color::Black => {
                let child = unsafe { pos.rebrand_stm_mut::<{ Color::White }>() };
                assert_hash_consistency_after_plies(child, plies - 1);
                child.unmake_move(mv);
            }
        }

        assert!(pos.is_zobrist_consistent());
    }

    #[test]
    fn with_zobrist_hash_remains_consistent_through_recursive_make_unmake() {
        let mut pos = PositionWithZobrist::<16, { Color::White }>::initial();
        assert_hash_consistency_after_plies(&mut pos, 5);
    }

    #[test]
    fn without_zobrist_make_unmake_round_trip() {
        let mut pos = PositionWithoutZobrist::<8, { Color::White }>::initial();
        let baseline = pos.clone();

        let mut moves = MoveList::new();
        pos.generate_moves(&mut moves);
        let mv = *moves.as_slice().first().expect("at least one legal move");

        pos.make_move(mv);
        let child = unsafe { pos.rebrand_stm_mut::<{ Color::Black }>() };
        child.unmake_move(mv);

        assert_eq!(pos, baseline);
        assert!(pos.is_zobrist_consistent());
    }
}
