//! Contains [`Position`], the main struct for representing a position in a chess game.

use super::bitboard::Bitboard;
use super::bitboard::BitboardUtils;
use super::board::Board;
use super::castling_rights::CastlingRights;
use super::color::Color;
use super::piece::Piece;
use super::position_context::PositionContext;
use super::square::Square;
use super::with_zobrist::WithZobrist;
use super::zobrist_policy::ZobristPolicy;
use crate::attacks::{multi_pawn_attacks, single_knight_attacks};
use crate::types::WithoutZobrist;
use std::fmt;

pub type PositionWithZobrist<const N: usize, const STM: Color> = Position<N, STM, WithZobrist>;

pub type PositionWithoutZobrist<const N: usize, const STM: Color> =
    Position<N, STM, WithoutZobrist>;

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
pub struct Position<const N: usize, const STM: Color, Z: ZobristPolicy = WithZobrist> {
    pub board: Board,
    pub halfmove: u16,
    pub(crate) contexts: [PositionContext<Z::HashState>; N],
    pub(crate) num_contexts: usize,
}

impl<const N: usize, const STM: Color, Z: ZobristPolicy> fmt::Debug for Position<N, STM, Z> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("board", &self.board)
            .field("side_to_move", &STM)
            .field("halfmove", &self.halfmove)
            .field("contexts", &&self.contexts[..self.num_contexts])
            .finish()
    }
}

impl<const N: usize, const STM: Color, Z: ZobristPolicy> PartialEq for Position<N, STM, Z> {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.halfmove == other.halfmove
            && self.num_contexts == other.num_contexts
            && self.contexts[..self.num_contexts] == other.contexts[..other.num_contexts]
    }
}

impl<const N: usize, const STM: Color, Z: ZobristPolicy> Eq for Position<N, STM, Z> {}

impl<const N: usize, const STM: Color, Z: ZobristPolicy> Position<N, STM, Z> {
    /// Builds a [`Position`] with a different const `STM` from the same fields (layout does not depend on `STM`).
    ///
    /// Only use when the underlying state already corresponds to side to move `NEXT` (for example after
    /// `make_move_in_place` or `unmake_move_in_place`).
    #[inline]
    pub fn rebrand_stm<const NEXT: Color>(self) -> Position<N, NEXT, Z> {
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

    /// Reinterprets a mutable reference to this position as having side-to-move `NEXT`.
    ///
    /// # Safety
    /// Caller must guarantee the underlying board/context already represent `NEXT` to move.
    #[inline]
    pub unsafe fn rebrand_stm_mut<const NEXT: Color>(&mut self) -> &mut Position<N, NEXT, Z> {
        // SAFETY: `Position` has identical layout for any `STM` value; only the type-level
        // side-to-move marker changes. Callers must uphold that runtime state matches `NEXT`.
        unsafe { &mut *(self as *mut Self).cast::<Position<N, NEXT, Z>>() }
    }

    /// Active context stack entries (root at index 0, current at `len - 1`).
    pub fn context_slice(&self) -> &[PositionContext<Z::HashState>] {
        &self.contexts[..self.num_contexts]
    }

    /// Number of contexts on the stack (always >= 1 when valid).
    pub fn num_contexts(&self) -> usize {
        self.num_contexts
    }

    /// Creates an initial state with the standard starting position (White to move).
    pub fn initial() -> Position<N, { Color::White }, Z> {
        debug_assert!(
            N >= 1,
            "Position context stack capacity N must be at least 1"
        );
        let board = Board::initial();
        let mut context = PositionContext::<Z::HashState>::blank();
        context.castling_rights = CastlingRights::B1111;
        context.zobrist_hash = Z::initial_hash(
            &board,
            context.castling_rights,
            context.double_pawn_push_file,
            Color::White,
        );
        let mut contexts = [PositionContext::<Z::HashState>::blank(); N];
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
    pub(crate) const fn king_square(&self, side: Color) -> Square {
        Square::from_bitboard(
            self.board.piece_mask::<{ Piece::King }>() & self.board.color_mask_at(side),
        )
        .expect("king present for side")
    }

    pub const fn context(&self) -> &PositionContext<Z::HashState> {
        debug_assert!(self.num_contexts > 0);
        &self.contexts[self.num_contexts - 1]
    }

    pub const fn mut_context(&mut self) -> &mut PositionContext<Z::HashState> {
        debug_assert!(self.num_contexts > 0);
        &mut self.contexts[self.num_contexts - 1]
    }

    pub const fn push_context(&mut self, context: PositionContext<Z::HashState>) {
        debug_assert!(self.num_contexts < N);
        self.contexts[self.num_contexts] = context;
        self.num_contexts += 1;
    }

    pub const fn pop_context(&mut self) -> PositionContext<Z::HashState> {
        debug_assert!(self.num_contexts > 1);
        let popped = self.contexts[self.num_contexts - 1];
        self.num_contexts -= 1;
        popped
    }

    pub(crate) const fn decrement_context_stack_for_unmake(&mut self) {
        debug_assert!(self.num_contexts > 1);
        self.num_contexts -= 1;
    }

    #[inline(always)]
    pub fn put_piece_at(&mut self, piece: Piece, square: Square) {
        self.board.put_piece_at(piece, square);
        Z::on_put_piece(&mut self.mut_context().zobrist_hash, piece, square);
    }

    #[inline(always)]
    pub fn put_piece_and_color(&mut self, color: Color, piece: Piece, square: Square) {
        self.board.put_piece_and_color(color, piece, square);
        Z::on_put_piece(&mut self.mut_context().zobrist_hash, piece, square);
    }

    #[inline(always)]
    pub fn remove_piece_at(&mut self, piece: Piece, square: Square) {
        self.board.remove_piece_at(piece, square);
        Z::on_remove_piece(&mut self.mut_context().zobrist_hash, piece, square);
    }

    #[inline(always)]
    pub fn remove_piece_and_color(&mut self, color: Color, piece: Piece, square: Square) {
        self.board.remove_piece_and_color(color, piece, square);
        Z::on_remove_piece(&mut self.mut_context().zobrist_hash, piece, square);
    }

    #[inline(always)]
    pub fn move_piece(&mut self, piece: Piece, from: Square, to: Square) {
        self.board.move_piece(piece, from, to);
        Z::on_move_piece(&mut self.mut_context().zobrist_hash, piece, from, to);
    }

    #[inline(always)]
    pub fn move_color(&mut self, color: Color, from: Square, to: Square) {
        self.board.move_color(color, from, to);
    }

    #[inline(always)]
    pub fn move_piece_and_color(&mut self, color: Color, piece: Piece, from: Square, to: Square) {
        self.board.move_piece_and_color(color, piece, from, to);
        Z::on_move_piece(&mut self.mut_context().zobrist_hash, piece, from, to);
    }

    #[inline(always)]
    pub fn set_castling_rights(&mut self, castling_rights: CastlingRights) {
        let context = self.mut_context();
        let old = context.castling_rights;
        context.castling_rights = castling_rights;
        Z::on_castling_rights_change(&mut context.zobrist_hash, old, castling_rights);
    }

    #[inline(always)]
    pub fn set_double_pawn_push_file(
        &mut self,
        double_pawn_push_file: crate::types::DoublePawnPushFile,
    ) {
        let context = self.mut_context();
        let old = context.double_pawn_push_file;
        context.double_pawn_push_file = double_pawn_push_file;
        Z::on_double_pawn_push_file_change(&mut context.zobrist_hash, old, double_pawn_push_file);
    }

    #[inline(always)]
    pub fn flip_side_to_move_hash(&mut self) {
        Z::on_side_to_move_flip(&mut self.mut_context().zobrist_hash);
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub const fn update_pins_and_checks(&mut self) {
        self.update_pins_and_checks_for_stm(STM);
    }

    /// Recomputes [`PositionContext::pinned`] / [`PositionContext::checkers`] for `stm` (must match the board).
    pub(crate) const fn update_pins_and_checks_for_stm(&mut self, side_to_move: Color) {
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

    pub const fn is_current_side_in_check(&self) -> bool {
        self.context().checkers != 0
    }

    pub fn is_insufficient_material<const USCF: bool>(&self) -> bool {
        self.board
            .are_both_sides_insufficient_material::<{ USCF }>()
    }

    pub const fn is_fifty_move_rule_reached(&self) -> bool {
        self.context().halfmove_clock >= 100
    }
}

#[cfg(test)]
mod state_tests {
    use super::Position;
    use crate::types::Color;

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
        use crate::types::MoveList;

        let mut pos = Position::<2, { Color::White }>::initial();
        let mut ml = MoveList::new();
        pos.generate_moves(&mut ml);
        let mv = *ml.as_slice().first().expect("at least one legal move");
        pos.make_move(mv);
        assert_eq!(pos.num_contexts(), 2);
        ml.clear();
        pos.generate_moves(&mut ml);
        let mv2 = *ml.as_slice().first().expect("at least one legal move");
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pos.make_move(mv2);
        }));
        assert!(
            r.is_err(),
            "second make_move with N=2 should panic in debug"
        );
    }
}
