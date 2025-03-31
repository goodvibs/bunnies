//! Contains the State struct, which is the main struct for representing a position in a chess game.

use crate::state::{Board, GameContext, GameResult};
use crate::{Bitboard, Color, PieceType};

/// A struct containing all the information needed to represent a position in a chess game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct GameState {
    pub board: Board,
    pub side_to_move: Color,
    pub halfmove: u16,
    pub result: GameResult,
    pub context: *mut GameContext,
}

impl GameState {
    /// Creates an initial state with the standard starting position.
    pub fn initial() -> GameState {
        let board = Board::initial();
        let zobrist_hash = board.zobrist_hash;
        
        let mut context = GameContext::initial();
        context.zobrist_hash = zobrist_hash;
        
        GameState {
            board,
            side_to_move: Color::White,
            halfmove: 0,
            result: GameResult::None,
            context: Box::into_raw(Box::new(context)),
        }
    }

    /// Gets the fullmove number of the position. 1-based.
    pub const fn get_fullmove(&self) -> u16 {
        self.halfmove / 2 + 1
    }

    pub fn current_side_attacks(&self) -> Bitboard {
        match unsafe { (*self.context).current_side_attacks.all } {
            0 => unsafe {
                (*self.context).current_side_attacks.update(&self.board);
                (*self.context).current_side_attacks.all
            }
            mask => mask
        }
    }

    pub fn opposite_side_attacks(&self) -> Bitboard {
        match unsafe { (*self.context).opposite_side_attacks.all } {
            0 => unsafe {
                (*self.context).opposite_side_attacks.update(&self.board);
                (*self.context).opposite_side_attacks.all
            }
            mask => mask
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
mod tests {
    use crate::state::{GameResult, GameState};
    use crate::utilities::print_bb;
    use crate::{Color, ColoredPieceType, GameContext, Move, MoveFlag, PieceType, Square};

    #[test]
    fn test_initial_state() {
        let state = GameState::initial();
        assert_eq!(state.side_to_move, Color::White);
        assert_eq!(state.halfmove, 0);
        assert_eq!(state.result, GameResult::None);
        assert_eq!(state.get_fullmove(), 1);
    }

    #[test]
    fn test_get_fullmove() {
        let mut state = GameState::initial();

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
    fn test_attacks() {
        let state = GameState::initial();
        assert_eq!(unsafe { (*state.context).current_side_attacks.clone() },
                   state.board.calc_attacks(Color::White));
        assert_eq!(unsafe { (*state.context).opposite_side_attacks.clone() },
                   state.board.calc_attacks(Color::Black));
        
        assert_eq!(state.current_side_attacks(), state.board.calc_attacks_mask(Color::White));
        assert_eq!(state.opposite_side_attacks(), state.board.calc_attacks_mask(Color::Black));
    }

    #[test]
    fn test_attacks_after_move() {
        let initial_state = GameState::initial();
        let next_state = {
            let mut state = initial_state.clone();
            let mv = Move::new_non_promotion(Square::E4, Square::E2, MoveFlag::NormalMove);
            state.make_move(mv);
            state
        };

        assert_eq!(unsafe { (*next_state.context).current_side_attacks.clone() },
                   next_state.board.calc_attacks(Color::Black));
        assert_eq!(unsafe { (*next_state.context).opposite_side_attacks.clone() },
                   next_state.board.calc_attacks(Color::White));
        
        assert_eq!(next_state.current_side_attacks(), next_state.board.calc_attacks_mask(Color::Black));
        assert_eq!(next_state.opposite_side_attacks(), next_state.board.calc_attacks_mask(Color::White));
    }
}
