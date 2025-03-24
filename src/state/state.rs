//! Contains the State struct, which is the main struct for representing a position in a chess game.

use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::{Bitboard, Color, PieceType};
use crate::state::{Board, GameContext, GameResult};

/// A struct containing all the information needed to represent a position in a chess game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct State {
    pub board: Board,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub result: GameResult,
    pub context: Rc<RefCell<GameContext>>,
}

impl State {
    /// Creates an initial state with the standard starting position.
    pub fn initial() -> State {
        let board = Board::initial();
        let zobrist_hash = board.zobrist_hash;
        State {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            result: GameResult::None,
            context: Rc::new(RefCell::new(GameContext::initial(zobrist_hash))),
        }
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn current_side_attacks(&self) -> Bitboard {
        match self.context.borrow().current_side_attacks {
            0 => self.board.calc_attacks_mask(self.side_to_move),
            attacks => attacks,
        }
    }

    pub fn opposite_side_attacks(&self) -> Bitboard {
        match &self.context.borrow().previous {
            // Some(previous) => previous.borrow().current_side_attacks,
            _ => self.board.calc_attacks_mask(self.side_to_move.flip())
        }
    }

    pub fn is_current_side_in_check(&self) -> bool {
        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let current_side_bb = self.board.color_masks[self.side_to_move as usize];
        kings_bb & current_side_bb & self.opposite_side_attacks() != 0
    }

    pub fn is_opposite_side_in_check(&self) -> bool {
        let kings_bb = self.board.piece_type_masks[PieceType::King as usize];
        let opposite_side_bb = self.board.color_masks[self.side_to_move.flip() as usize];
        kings_bb & opposite_side_bb & self.current_side_attacks() != 0
    }

    pub fn update_insufficient_material(&mut self, use_uscf_rules: bool) {
        if self.board.are_both_sides_insufficient_material(use_uscf_rules) {
            self.result = GameResult::InsufficientMaterial;
        }
    }

    pub fn update_halfmove_clock(&mut self) {
        if self.context.borrow().halfmove_clock < 100 {
            self.result = GameResult::FiftyMoveRule;
        }
    }

    pub fn update_threefold_repetition(&mut self) {
        if self.context.borrow().has_threefold_repetition_occurred() {
            self.result = GameResult::ThreefoldRepetition;
        }
    }
}

#[cfg(test)]
mod state_tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::utils::{Color, ColoredPiece, PieceType, Square};
    use crate::state::{GameContext, GameResult, State};
    use crate::utils::charboard::print_bb;

    #[test]
    fn test_initial_state() {
        let state = State::initial();
        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.halfmove, 0);
        assert_eq!(state.result, GameResult::None);
        assert_eq!(state.get_fullmove(), 1);
    }

    #[test]
    fn test_get_fullmove() {
        let mut state = State::initial();

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
        let state = State::initial();
        let expected_attacks = state.board.calc_attacks_mask(state.side_to_move);
        print_bb(expected_attacks);
        assert_eq!(state.current_side_attacks(), expected_attacks);
    }

    #[test]
    fn test_opposite_side_attacks() {
        let initial_state = State::initial();
        let initial_black_attacks = initial_state.board.calc_attacks_mask(initial_state.side_to_move.flip());
        assert_eq!(initial_state.opposite_side_attacks(), initial_black_attacks);
        initial_state.context.borrow_mut().initialize_current_side_attacks(initial_black_attacks);
        
        let mut next_state_board = initial_state.board.clone();
        next_state_board.move_colored_piece(
            ColoredPiece::from(Color::White, PieceType::Pawn),
            Square::E4,
            Square::E2
        );
        let next_state_zobrist = next_state_board.zobrist_hash;
        
        
        let next_state_context = GameContext::new_with_previous(
            &initial_state.context,
            next_state_zobrist,
            0
        );
        
        let next_state = State {
            board: next_state_board,
            side_to_move: Color::Black,
            halfmove: 1,
            result: GameResult::None,
            context: Rc::new(RefCell::new(next_state_context)),
        };
        
        let next_state_white_attacks = next_state.board.calc_attacks_mask(next_state.side_to_move.flip());
        assert_eq!(next_state.opposite_side_attacks(), next_state_white_attacks);
    }
}