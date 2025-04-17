//! Contains the State struct, which is the main struct for representing a position in a chess game.

use crate::position::{Board, PositionContext, GameResult};
use crate::{Bitboard, BitboardUtils, Color, PieceType, Square};
use crate::masks::{RANK_1, RANK_8};

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
        let zobrist_hash = board.zobrist_hash;
        Position {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            result: GameResult::None,
            context: Box::into_raw(Box::new(PositionContext::initial(zobrist_hash))),
        }
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn current_side_attacks(&self) -> Bitboard {
        match unsafe { (*self.context).current_side_attacks } {
            0 => self.board.calc_attacks_mask(self.side_to_move),
            attacks => attacks,
        }
    }

    pub fn opposite_side_attacks(&self) -> Bitboard {
        match unsafe { &(*self.context).previous } {
            // Some(previous) => previous.borrow().current_side_attacks,
            _ => self.board.calc_attacks_mask(self.side_to_move.other()),
        }
    }

    pub fn is_current_side_in_check(&self) -> bool {
        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let current_side_bb = self.board.color_masks[self.side_to_move as usize];
        kings_bb & current_side_bb & self.opposite_side_attacks() != 0
    }

    pub fn is_opposite_side_in_check(&self) -> bool {
        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let opposite_side_bb = self.board.color_masks[self.side_to_move.other() as usize];
        kings_bb & opposite_side_bb & self.current_side_attacks() != 0
    }

    pub fn update_insufficient_material(&mut self, use_uscf_rules: bool) {
        if self
            .board
            .are_both_sides_insufficient_material(use_uscf_rules)
        {
            self.result = GameResult::InsufficientMaterial;
        }
    }

    pub fn update_halfmove_clock(&mut self) {
        if unsafe { (*self.context).halfmove_clock < 100 } {
            self.result = GameResult::FiftyMoveRule;
        }
    }

    pub fn update_threefold_repetition(&mut self) {
        if unsafe { (*self.context).has_threefold_repetition_occurred() } {
            self.result = GameResult::ThreefoldRepetition;
        }
    }

    pub fn update_pinned_pieces(&self) {
        let current_side_king = self.current_side_king();
        let current_side_king_square = unsafe { Square::from_bitboard(current_side_king) };

        let unsafe_diagonals = current_side_king_square.diagonals_mask();
        let unsafe_orthogonals = current_side_king_square.orthogonals_mask();
        
        let possible_diagonal_pinners = (self.opposite_side_bishops() | self.opposite_side_queens())
            & unsafe_diagonals;
        let possible_orthogonal_pinners = (self.opposite_side_rooks() | self.opposite_side_queens())
            & unsafe_orthogonals;
        let possible_pinners = possible_diagonal_pinners | possible_orthogonal_pinners;

        let mut pinned = 0;

        for possible_pinner in possible_pinners.iter_set_bits_as_squares() {
            let blockers_bb = Bitboard::between(current_side_king_square, possible_pinner);
            if blockers_bb != 0 && blockers_bb.count_ones() == 1 {
                pinned |= blockers_bb;
            }
        }
        
        pinned &= self.current_side_pieces();

        unsafe { (*self.context).pinned_pieces = pinned; }
    }
    
    pub fn pinned_pieces(&self) -> Bitboard {
        unsafe { (*self.context).pinned_pieces }
    }
    
    pub fn current_side_pieces(&self) -> Bitboard {
        self.board.color_masks[self.side_to_move as usize]
    }
    
    pub fn current_side_pawns(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Pawn as usize] &
            self.current_side_pieces()
    }

    pub fn current_side_knights(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Knight as usize] &
            self.current_side_pieces()
    }

    pub fn current_side_bishops(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Bishop as usize] &
            self.current_side_pieces()
    }

    pub fn current_side_rooks(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Rook as usize] &
            self.current_side_pieces()
    }

    pub fn current_side_queens(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Queen as usize] &
            self.current_side_pieces()
    }

    pub fn current_side_king(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::King as usize] &
            self.current_side_pieces()
    }

    pub fn opposite_side_pieces(&self) -> Bitboard {
        self.board.color_masks[self.side_to_move.other() as usize]
    }

    pub fn opposite_side_pawns(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Pawn as usize] &
            self.opposite_side_pieces()
    }

    pub fn opposite_side_knights(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Knight as usize] &
            self.opposite_side_pieces()
    }

    pub fn opposite_side_bishops(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Bishop as usize] &
            self.opposite_side_pieces()
    }

    pub fn opposite_side_rooks(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Rook as usize] &
            self.opposite_side_pieces()
    }

    pub fn opposite_side_queens(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::Queen as usize] &
            self.opposite_side_pieces()
    }

    pub fn opposite_side_king(&self) -> Bitboard {
        self.board.piece_type_masks[PieceType::King as usize] &
            self.opposite_side_pieces()
    }
    
    pub const fn current_side_promotion_rank_mask(&self) -> Bitboard {
        match self.side_to_move {
            Color::White => RANK_8,
            Color::Black => RANK_1
        }
    }

    pub const fn opposite_side_promotion_rank_mask(&self) -> Bitboard {
        match self.side_to_move.other() {
            Color::White => RANK_8,
            Color::Black => RANK_1
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
    use crate::position::{PositionContext, GameResult, Position};
    use crate::{Color, ColoredPieceType, PieceType, Square};

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

    #[test]
    fn test_current_side_attacks() {
        let state = Position::initial();
        let expected_attacks = state.board.calc_attacks_mask(state.side_to_move);
        assert_eq!(state.current_side_attacks(), expected_attacks);
    }

    #[test]
    fn test_opposite_side_attacks() {
        let initial_state = Position::initial();
        let initial_black_attacks = initial_state
            .board
            .calc_attacks_mask(initial_state.side_to_move.other());
        assert_eq!(initial_state.opposite_side_attacks(), initial_black_attacks);
        unsafe { (*initial_state.context).initialize_current_side_attacks(initial_black_attacks) };

        let mut next_state_board = initial_state.board.clone();
        next_state_board.move_colored_piece(
            ColoredPieceType::new(Color::White, PieceType::Pawn),
            Square::E4,
            Square::E2,
        );
        let next_state_zobrist = next_state_board.zobrist_hash;

        let next_state_context =
            unsafe { PositionContext::new_with_previous(initial_state.context, next_state_zobrist, 0) };

        let next_state = Position {
            board: next_state_board,
            side_to_move: Color::Black,
            halfmove: 1,
            result: GameResult::None,
            context: Box::into_raw(Box::new(next_state_context)),
        };

        let next_state_white_attacks = next_state
            .board
            .calc_attacks_mask(next_state.side_to_move.other());
        assert_eq!(next_state.opposite_side_attacks(), next_state_white_attacks);
    }
}
