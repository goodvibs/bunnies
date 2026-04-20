//! Contains [`Position`], the main struct for representing a position in a chess game.

use crate::attacks::{multi_pawn_attacks, single_knight_attacks};
use crate::position::{Board, GameResult, PositionContext};
use crate::{Bitboard, BitboardUtils, CastlingRights, Color, ConstBitboardGeometry, Piece, Square};
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
    pub result: GameResult,
    pub(crate) contexts: [PositionContext; N],
    pub(crate) context_len: usize,
}

impl<const N: usize, const STM: Color> fmt::Debug for Position<N, STM> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("board", &self.board)
            .field("side_to_move", &STM)
            .field("halfmove", &self.halfmove)
            .field("result", &self.result)
            .field("contexts", &&self.contexts[..self.context_len])
            .finish()
    }
}

impl<const N: usize, const STM: Color> PartialEq for Position<N, STM> {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.halfmove == other.halfmove
            && self.result == other.result
            && self.context_len == other.context_len
            && self.contexts[..self.context_len] == other.contexts[..other.context_len]
    }
}

impl<const N: usize, const STM: Color> Eq for Position<N, STM> {}

impl<const N: usize, const STM: Color> Position<N, STM> {
    #[inline]
    pub const fn side_to_move(&self) -> Color {
        STM
    }

    /// Builds a [`Position`] with a different const `STM` from the same fields (layout does not depend on `STM`).
    ///
    /// Only use when the underlying state already corresponds to side to move `NEXT` (for example after
    /// `make_move_in_place` or `unmake_move_in_place`).
    #[inline]
    pub(crate) fn rebrand_stm<const NEXT: Color>(self) -> Position<N, NEXT> {
        let Position {
            board,
            halfmove,
            result,
            contexts,
            context_len,
        } = self;
        Position {
            board,
            halfmove,
            result,
            contexts,
            context_len,
        }
    }

    /// Active context stack entries (root at index 0, current at `len - 1`).
    pub fn context_slice(&self) -> &[PositionContext] {
        &self.contexts[..self.context_len]
    }

    /// Number of contexts on the stack (always >= 1 when valid).
    pub fn context_len(&self) -> usize {
        self.context_len
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
            result: GameResult::None,
            contexts,
            context_len: 1,
        };
        res.update_pins_and_checks();
        debug_assert!(res.is_unequivocally_valid());

        res
    }

    pub fn context(&self) -> &PositionContext {
        debug_assert!(self.context_len > 0);
        &self.contexts[self.context_len - 1]
    }

    pub fn mut_context(&mut self) -> &mut PositionContext {
        debug_assert!(self.context_len > 0);
        &mut self.contexts[self.context_len - 1]
    }

    pub fn push_context(&mut self, context: PositionContext) {
        debug_assert!(self.context_len < N);
        self.contexts[self.context_len] = context;
        self.context_len += 1;
    }

    pub fn pop_context(&mut self) -> PositionContext {
        debug_assert!(self.context_len > 1);
        let popped = self.contexts[self.context_len - 1];
        self.context_len -= 1;
        popped
    }

    pub(crate) fn decrement_context_stack_for_unmake(&mut self) {
        debug_assert!(self.context_len > 1);
        self.context_len -= 1;
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn update_pins_and_checks(&mut self) {
        self.update_pins_and_checks_for_stm(STM);
    }

    /// Recomputes [`PositionContext::pinned`] / [`PositionContext::checkers`] for `stm` (must match the board).
    pub(crate) fn update_pins_and_checks_for_stm(&mut self, stm: Color) {
        let opp = stm.other();

        let current_side_king =
            self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(stm);

        if current_side_king.count_ones() != 1 {
            return;
        }

        let current_side_king_square = Square::from_bitboard(current_side_king).expect("king mask");

        let relevant_diagonals = current_side_king_square.diagonals_mask();
        let relevant_orthogonals = current_side_king_square.orthogonals_mask();

        let opp_bb = self.board.color_mask_at(opp);
        let relevant_diagonal_attackers = (self.board.piece_mask::<{ Piece::Bishop }>()
            | self.board.piece_mask::<{ Piece::Queen }>())
            & opp_bb
            & relevant_diagonals;
        let relevant_orthogonal_attackers = (self.board.piece_mask::<{ Piece::Rook }>()
            | self.board.piece_mask::<{ Piece::Queen }>())
            & opp_bb
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

        pinned &= self.board.color_mask_at(stm);

        checkers |= single_knight_attacks(current_side_king_square)
            & self.board.piece_mask::<{ Piece::Knight }>()
            & opp_bb;
        checkers |= multi_pawn_attacks(current_side_king, stm)
            & self.board.piece_mask::<{ Piece::Pawn }>()
            & opp_bb;

        let context = self.mut_context();
        context.pinned = pinned;
        context.checkers = checkers;
    }

    pub fn is_current_side_in_check(&self) -> bool {
        self.context().checkers != 0
    }

    pub fn update_insufficient_material(&mut self, use_uscf_rules: bool) {
        if self
            .board
            .are_both_sides_insufficient_material(use_uscf_rules)
        {
            self.result = GameResult::InsufficientMaterial;
        }
    }

    pub fn update_fifty_move_rule(&mut self) {
        if self.context().halfmove_clock < 100 {
            self.result = GameResult::FiftyMoveRule;
        }
    }
}

#[cfg(test)]
mod state_tests {
    use crate::Color;
    use crate::position::Position;

    #[test]
    fn test_initial_state() {
        let state = Position::<1, { Color::White }>::initial();
        assert_eq!(state.side_to_move(), Color::White);
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
        assert_eq!(pos.context_len(), 2);
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
