//! Contains the State struct, which is the main struct for representing a position in a chess game.

use crate::attacks::{multi_pawn_attacks, single_knight_attacks};
use crate::position::{Board, GameResult, PositionContext};
use crate::{Bitboard, BitboardUtils, Color, Piece, Square};

/// A struct containing all the information needed to represent a position in a chess game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Position {
    pub board: Board,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub result: GameResult,
    pub context: *mut PositionContext,
}

impl Position {
    /// Creates an initial state with the standard starting position.
    pub fn initial() -> Position {
        let board = Board::initial();
        let mut context = PositionContext::initial();
        context.zobrist_hash = board.zobrist_hash;
        let mut res = Position {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            result: GameResult::None,
            context: Box::into_raw(Box::new(context)),
        };
        res.update_pins_and_checks();
        assert!(res.is_unequivocally_valid());

        res
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn context(&self) -> &PositionContext {
        unsafe { &(*self.context) }
    }

    pub fn mut_context(&mut self) -> &mut PositionContext {
        unsafe { &mut (*self.context) }
    }

    pub fn update_pins_and_checks(&mut self) {
        let current_side_king = self.current_side_king();

        if current_side_king.count_ones() != 1 {
            return;
        }

        let current_side_king_square = unsafe { Square::from_bitboard(current_side_king) };

        let relevant_diagonals = current_side_king_square.diagonals_mask();
        let relevant_orthogonals = current_side_king_square.orthogonals_mask();

        let relevant_diagonal_attackers =
            (self.opposite_side_bishops() | self.opposite_side_queens()) & relevant_diagonals;
        let relevant_orthogonal_attackers =
            (self.opposite_side_rooks() | self.opposite_side_queens()) & relevant_orthogonals;
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

        pinned &= self.current_side_pieces();

        checkers |= single_knight_attacks(current_side_king_square) & self.opposite_side_knights();
        checkers |=
            multi_pawn_attacks(current_side_king, self.side_to_move) & self.opposite_side_pawns();

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

    pub fn update_threefold_repetition(&mut self) {
        if self.context().has_threefold_repetition_occurred() {
            self.result = GameResult::ThreefoldRepetition;
        }
    }

    pub const fn current_side_pieces(&self) -> Bitboard {
        self.board.color_masks[self.side_to_move as usize]
    }

    pub const fn current_side_pawns(&self) -> Bitboard {
        self.board.piece_masks[Piece::Pawn as usize] & self.current_side_pieces()
    }

    pub const fn current_side_knights(&self) -> Bitboard {
        self.board.piece_masks[Piece::Knight as usize] & self.current_side_pieces()
    }

    pub const fn current_side_bishops(&self) -> Bitboard {
        self.board.piece_masks[Piece::Bishop as usize] & self.current_side_pieces()
    }

    pub const fn current_side_rooks(&self) -> Bitboard {
        self.board.piece_masks[Piece::Rook as usize] & self.current_side_pieces()
    }

    pub const fn current_side_queens(&self) -> Bitboard {
        self.board.piece_masks[Piece::Queen as usize] & self.current_side_pieces()
    }

    pub const fn current_side_king(&self) -> Bitboard {
        self.board.piece_masks[Piece::King as usize] & self.current_side_pieces()
    }

    pub const fn opposite_side_pieces(&self) -> Bitboard {
        self.board.color_masks[self.side_to_move.other() as usize]
    }

    pub const fn opposite_side_pawns(&self) -> Bitboard {
        self.board.piece_masks[Piece::Pawn as usize] & self.opposite_side_pieces()
    }

    pub const fn opposite_side_knights(&self) -> Bitboard {
        self.board.piece_masks[Piece::Knight as usize] & self.opposite_side_pieces()
    }

    pub const fn opposite_side_bishops(&self) -> Bitboard {
        self.board.piece_masks[Piece::Bishop as usize] & self.opposite_side_pieces()
    }

    pub const fn opposite_side_rooks(&self) -> Bitboard {
        self.board.piece_masks[Piece::Rook as usize] & self.opposite_side_pieces()
    }

    pub const fn opposite_side_queens(&self) -> Bitboard {
        self.board.piece_masks[Piece::Queen as usize] & self.opposite_side_pieces()
    }

    pub const fn opposite_side_king(&self) -> Bitboard {
        self.board.piece_masks[Piece::King as usize] & self.opposite_side_pieces()
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

// impl Drop for State {
//     fn drop(&mut self) {
//         unsafe {
//             let mut context_ptr = self.context;
//             while let Some(previous) = (*context_ptr).previous {
//                 let _ = Box::from_raw(context_ptr);
//                 context_ptr = previous;
//             }
//             // let _ = Box::from_raw(context_ptr);
//         }
//     }
// }

#[cfg(test)]
mod state_tests {
    use crate::Color;
    use crate::position::{GameResult, Position};

    #[test]
    fn test_initial_state() {
        let state = Position::initial();
        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.halfmove, 0);
        assert_eq!(state.result, GameResult::None);
        assert_eq!(state.get_fullmove(), 1);
    }

    #[test]
    fn test_get_fullmove() {
        let mut state = Position::initial();

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
}
