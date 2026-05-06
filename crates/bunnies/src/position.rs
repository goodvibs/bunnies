//! Contains [`Position`], the main struct for representing a position in a chess game.

use crate::attacks::{multi_pawn_attacks, single_knight_attacks};
use crate::{
    Bitboard, BitboardUtils, Board, CastlingRights, Color, ConstBitboardGeometry, Piece,
    PositionContext, Square,
};
use std::fmt;

/// Chess position with a fixed-size context stack of capacity `N` (root plus at most `N - 1` plies).
///
/// `STM` is the **side to move**, encoded as a const generic [`Color`] (`Color::White` or
/// `Color::Black`) so it is known at compile time.
///
/// `N` must be at least **1**. Choose `N` at compile time so the deepest `make_move` / `unmake_move`
/// chain you use (search depth, PGN main line length, etc.) never needs more than **`N` context
/// slots** (including the root). Pushing beyond that is a **contract violation**: debug builds
/// panic on `debug_assert!`; release builds may exhibit **undefined behavior** (out-of-bounds write).
#[derive(Clone)]
pub struct Position<const N: usize, const STM: Color> {
    pub board: Board,
    pub halfmove: u16,
    pub(crate) contexts: [PositionContext; N],
    pub(crate) num_contexts: usize,
}

impl<const N: usize, const STM: Color> fmt::Debug for Position<N, STM> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("board", &self.board)
            .field("side_to_move", &STM)
            .field("halfmove", &self.halfmove)
            .field("contexts", &&self.contexts[..self.num_contexts])
            .finish()
    }
}

impl<const N: usize, const STM: Color> PartialEq for Position<N, STM> {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.halfmove == other.halfmove
            && self.num_contexts == other.num_contexts
            && self.contexts[..self.num_contexts] == other.contexts[..other.num_contexts]
    }
}

impl<const N: usize, const STM: Color> Eq for Position<N, STM> {}

impl<const N: usize, const STM: Color> Position<N, STM> {
    /// Builds a [`Position`] with a different const `STM` from the same fields (layout does not depend on `STM`).
    ///
    /// Only use when the underlying state already corresponds to side to move `NEXT` (for example after
    /// `make_move_in_place` or `unmake_move_in_place`).
    #[inline]
    pub fn rebrand_stm<const NEXT: Color>(self) -> Position<N, NEXT> {
        let Position {
            board,
            halfmove,
            contexts,
            num_contexts,
        } = self;
        Position {
            board,
            halfmove,
            contexts,
            num_contexts,
        }
    }

    /// Active context stack entries (root at index 0, current at `len - 1`).
    pub fn context_slice(&self) -> &[PositionContext] {
        &self.contexts[..self.num_contexts]
    }

    /// Number of contexts on the stack (always >= 1 when valid).
    pub fn num_contexts(&self) -> usize {
        self.num_contexts
    }

    /// Creates an initial state with the standard starting position (White to move).
    pub fn initial() -> Position<N, { Color::White }> {
        debug_assert!(
            N >= 1,
            "Position context stack capacity N must be at least 1"
        );
        let board = Board::initial();
        let mut context = PositionContext::blank();
        context.castling_rights = CastlingRights::ALL;
        context.zobrist_hash = crate::calc_zobrist_hash(&board);
        let mut contexts = [PositionContext::blank(); N];
        contexts[0] = context;
        let mut res = Position {
            board,
            halfmove: 0,
            contexts,
            num_contexts: 1,
        };
        res.update_pins_and_checks();
        debug_assert!(res.is_unequivocally_valid());

        res
    }

    /// Square of the king for `side`. Legal positions have exactly one such king.
    #[inline]
    pub(crate) fn king_square(&self, side: Color) -> Square {
        Square::from_bitboard(
            self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(side),
        )
        .expect("king present for side")
    }

    pub fn context(&self) -> &PositionContext {
        debug_assert!(self.num_contexts > 0);
        &self.contexts[self.num_contexts - 1]
    }

    pub fn mut_context(&mut self) -> &mut PositionContext {
        debug_assert!(self.num_contexts > 0);
        &mut self.contexts[self.num_contexts - 1]
    }

    pub fn push_context(&mut self, context: PositionContext) {
        debug_assert!(self.num_contexts < N);
        self.contexts[self.num_contexts] = context;
        self.num_contexts += 1;
    }

    pub fn pop_context(&mut self) -> PositionContext {
        debug_assert!(self.num_contexts > 1);
        let popped = self.contexts[self.num_contexts - 1];
        self.num_contexts -= 1;
        popped
    }

    pub(crate) fn decrement_context_stack_for_unmake(&mut self) {
        debug_assert!(self.num_contexts > 1);
        self.num_contexts -= 1;
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn update_pins_and_checks(&mut self) {
        self.update_pins_and_checks_for_stm(STM);
    }

    /// Recomputes [`PositionContext::pinned`] / [`PositionContext::checkers`] for `stm` (must match the board).
    pub(crate) fn update_pins_and_checks_for_stm(&mut self, side_to_move: Color) {
        let opponent = side_to_move.other();

        let current_side_king_mask =
            self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(side_to_move);

        if current_side_king_mask.count_ones() != 1 {
            return;
        }

        let current_side_king_square = self.king_square(side_to_move);

        let relevant_diagonals = current_side_king_square.diagonals_mask();
        let relevant_orthogonals = current_side_king_square.orthogonals_mask();

        let opponent_mask = self.board.color_mask_at(opponent);
        let relevant_diagonal_attackers = (self.board.piece_mask::<{ Piece::Bishop }>()
            | self.board.piece_mask::<{ Piece::Queen }>())
            & opponent_mask
            & relevant_diagonals;
        let relevant_orthogonal_attackers = (self.board.piece_mask::<{ Piece::Rook }>()
            | self.board.piece_mask::<{ Piece::Queen }>())
            & opponent_mask
            & relevant_orthogonals;
        let relevant_sliding_attackers =
            relevant_diagonal_attackers | relevant_orthogonal_attackers;

        let mut pinned = 0;
        let mut checkers = 0;

        let occupied = self.board.pieces();

        for attacker_square in relevant_sliding_attackers.iter_set_bits_as_squares() {
            let blockers = Bitboard::between(current_side_king_square, attacker_square) & occupied;

            if blockers == 0 {
                checkers |= attacker_square.mask();
            } else if blockers.count_ones() == 1 {
                pinned |= blockers;
            }
        }

        pinned &= self.board.color_mask_at(side_to_move);

        checkers |= single_knight_attacks(current_side_king_square)
            & self.board.piece_mask::<{ Piece::Knight }>()
            & opponent_mask;
        checkers |= multi_pawn_attacks(current_side_king_mask, side_to_move)
            & self.board.piece_mask::<{ Piece::Pawn }>()
            & opponent_mask;

        let context = self.mut_context();
        context.pinned = pinned;
        context.checkers = checkers;
    }

    pub fn is_current_side_in_check(&self) -> bool {
        self.context().checkers != 0
    }

    pub fn is_insufficient_material(&self, use_uscf_rules: bool) -> bool {
        self.board
            .are_both_sides_insufficient_material(use_uscf_rules)
    }

    pub fn is_fifty_move_rule_reached(&self) -> bool {
        self.context().halfmove_clock >= 100
    }
}

#[cfg(test)]
mod state_tests {
    use crate::Color;
    use crate::Position;

    #[test]
    fn test_initial_state() {
        let state = Position::<1, { Color::White }>::initial();
        assert_eq!(state.halfmove, 0);
        assert_eq!(state.get_fullmove(), 1);
    }

    #[test]
    fn test_get_fullmove() {
        let mut state = Position::<1, { Color::White }>::initial();

        assert_eq!(state.get_fullmove(), 1);

        state.halfmove = 1;
        assert_eq!(state.get_fullmove(), 1);

        state.halfmove = 2;
        assert_eq!(state.get_fullmove(), 2);

        state.halfmove = 3;
        assert_eq!(state.get_fullmove(), 2);

        state.halfmove = 10;
        assert_eq!(state.get_fullmove(), 6);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_context_stack_overflow_second_move_panics_in_debug() {
        use crate::MoveList;

        let mut pos = Position::<2, { Color::White }>::initial();
        let mut ml = MoveList::new();
        pos.generate_legal_moves(&mut ml);
        let mv = *ml.as_slice().first().expect("at least one legal move");
        pos.make_move(mv);
        assert_eq!(pos.num_contexts(), 2);
        ml.clear();
        pos.generate_legal_moves(&mut ml);
        let mv2 = *ml.as_slice().first().expect("at least one legal move");
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = pos.make_move(mv2);
        }));
        assert!(
            r.is_err(),
            "second make_move with N=2 should panic in debug"
        );
    }
}
