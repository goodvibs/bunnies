//! Contains [`Position`], the main struct for representing a position in a chess game.

use crate::attacks::{multi_pawn_attacks, single_knight_attacks};
use crate::position::{Board, GameResult, PositionContext};
use crate::{Bitboard, BitboardUtils, Color, Piece, Square};
use std::fmt;

/// Error from [`Position::make_move`] when the context stack is full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionError {
    ContextStackFull,
}

/// Chess position with a fixed-size context stack of capacity `N` (root plus at most `N - 1` plies).
///
/// `N` must be at least **1**. Choose `N` at compile time so the deepest `make_move` chain you use
/// (search depth, PGN main line length, etc.) never exceeds `N` slots (including the root context).
#[derive(Clone)]
pub struct Position<const N: usize> {
    pub board: Board,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub result: GameResult,
    pub(crate) contexts: [PositionContext; N],
    pub(crate) context_len: usize,
}

impl<const N: usize> fmt::Debug for Position<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Position")
            .field("board", &self.board)
            .field("side_to_move", &self.side_to_move)
            .field("halfmove", &self.halfmove)
            .field("result", &self.result)
            .field("contexts", &&self.contexts[..self.context_len])
            .finish()
    }
}

impl<const N: usize> PartialEq for Position<N> {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.side_to_move == other.side_to_move
            && self.halfmove == other.halfmove
            && self.result == other.result
            && self.context_len == other.context_len
            && self.contexts[..self.context_len] == other.contexts[..other.context_len]
    }
}

impl<const N: usize> Eq for Position<N> {}

impl<const N: usize> Position<N> {
    /// Active context stack entries (root at index 0, current at `len - 1`).
    pub fn context_slice(&self) -> &[PositionContext] {
        &self.contexts[..self.context_len]
    }

    /// Number of contexts on the stack (always >= 1 when valid).
    pub fn context_len(&self) -> usize {
        self.context_len
    }

    /// Creates an initial state with the standard starting position.
    pub fn initial() -> Self {
        assert!(
            N >= 1,
            "Position context stack capacity N must be at least 1"
        );
        let board = Board::initial();
        let mut context = PositionContext::blank();
        context.castling_rights = 0b00001111;
        context.zobrist_hash = crate::calc_zobrist_hash(&board);
        let mut contexts = [PositionContext::blank(); N];
        contexts[0] = context;
        let mut res = Self {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            result: GameResult::None,
            contexts,
            context_len: 1,
        };
        res.update_pins_and_checks();
        assert!(res.is_unequivocally_valid());

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

    pub fn try_push_context(&mut self, context: PositionContext) -> Result<(), PositionError> {
        if self.context_len >= N {
            return Err(PositionError::ContextStackFull);
        }
        self.contexts[self.context_len] = context;
        self.context_len += 1;
        Ok(())
    }

    pub fn pop_context(&mut self) -> PositionContext {
        assert!(self.context_len > 1);
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
        let stm = self.side_to_move;
        let opp = stm.other();

        let current_side_king = self.board.piece_mask(Piece::King) & self.board.color_mask(stm);

        if current_side_king.count_ones() != 1 {
            return;
        }

        let current_side_king_square = unsafe { Square::from_bitboard(current_side_king) };

        let relevant_diagonals = current_side_king_square.diagonals_mask();
        let relevant_orthogonals = current_side_king_square.orthogonals_mask();

        let opp_bb = self.board.color_mask(opp);
        let relevant_diagonal_attackers = (self.board.piece_mask(Piece::Bishop)
            | self.board.piece_mask(Piece::Queen))
            & opp_bb
            & relevant_diagonals;
        let relevant_orthogonal_attackers = (self.board.piece_mask(Piece::Rook)
            | self.board.piece_mask(Piece::Queen))
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

        pinned &= self.board.color_mask(stm);

        checkers |= single_knight_attacks(current_side_king_square)
            & self.board.piece_mask(Piece::Knight)
            & opp_bb;
        checkers |= multi_pawn_attacks(current_side_king, stm)
            & self.board.piece_mask(Piece::Pawn)
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

    pub const fn current_side_promotion_rank(&self) -> u8 {
        match self.side_to_move {
            Color::White => 7,
            Color::Black => 0,
        }
    }

    pub const fn opposite_side_promotion_rank(&self) -> u8 {
        match self.side_to_move.other() {
            Color::White => 7,
            Color::Black => 0,
        }
    }
}

#[cfg(test)]
mod state_tests {
    use crate::Color;
    use crate::position::{GameResult, Position};

    #[test]
    fn test_initial_state() {
        let state = Position::<1>::initial();
        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.halfmove, 0);
        assert_eq!(state.result, GameResult::None);
        assert_eq!(state.get_fullmove(), 1);
    }

    #[test]
    fn test_get_fullmove() {
        let mut state = Position::<1>::initial();

        assert_eq!(state.get_fullmove(), 1); // Initial position

        state.halfmove = 1;
        assert_eq!(state.get_fullmove(), 1); // After 1 halfmove

        state.halfmove = 2;
        assert_eq!(state.get_fullmove(), 2); // After 2 halfmoves

        state.halfmove = 3;
        assert_eq!(state.get_fullmove(), 2); // After 3 halfmoves

        state.halfmove = 10;
        assert_eq!(state.get_fullmove(), 6); // After 10 halfmoves
    }

    #[test]
    fn test_context_stack_full_returns_error() {
        use crate::position::PositionError;
        let mut pos = Position::<2>::initial();
        let mv = pos
            .moves()
            .into_iter()
            .next()
            .expect("at least one legal move");
        pos.make_move(mv).unwrap();
        assert_eq!(pos.context_len(), 2);
        let mv2 = pos
            .moves()
            .into_iter()
            .next()
            .expect("at least one legal move");
        assert_eq!(pos.make_move(mv2), Err(PositionError::ContextStackFull));
    }
}
