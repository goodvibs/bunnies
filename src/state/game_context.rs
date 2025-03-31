//! Context struct and methods

use crate::{Bitboard, Color};
use crate::PieceType;
use crate::state::attacks_by_color::AttacksByColor;

/// A struct containing metadata about the current and past states of the game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct GameContext {
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8,  // 0, 0, 0, 0, wk, wq, bk, bq

    // updated after every move
    pub captured_piece: PieceType,
    pub previous: Option<*mut GameContext>,
    pub zobrist_hash: Bitboard,
    pub current_side_attacks: AttacksByColor,
    pub opposite_side_attacks: AttacksByColor
}

impl GameContext {
    /// Creates a new context linking to the previous context.
    /// The created context must be updated further to be valid.
    pub unsafe fn new_with_previous(previous_context: *mut GameContext) -> GameContext {
        let (previous_halfmove_clock, previous_castling_rights, current_side_attacks_prediction, opposite_side_attacks_prediction) = {
            let previous_context = unsafe { &*previous_context };

            assert_ne!(
                previous_context.current_side_attacks.all, 
                0,
                "Previous context must have updated current_side_attacks"
            );
            
            assert_ne!(
                previous_context.opposite_side_attacks.all, 
                0,
                "Previous context must have updated opposite_side_attacks"
            );
            
            assert_ne!(
                previous_context.zobrist_hash,
                0,
                "Previous context must have an updated zobrist_hash"
            );
            
            (
                previous_context.halfmove_clock,
                previous_context.castling_rights,
                // previous_context.opposite_side_attacks.clone(),
                // previous_context.current_side_attacks.clone()
                AttacksByColor::new(previous_context.opposite_side_attacks.side),
                AttacksByColor::new(previous_context.current_side_attacks.side)
            )
        };
        
        GameContext {
            halfmove_clock: previous_halfmove_clock + 1,
            double_pawn_push: -1,
            castling_rights: previous_castling_rights,
            captured_piece: PieceType::NoPieceType,
            previous: Some(previous_context),
            zobrist_hash: 0,
            current_side_attacks: current_side_attacks_prediction,
            opposite_side_attacks: opposite_side_attacks_prediction
        }
    }

    /// Creates a new context with no previous context.
    /// This is used for the initial position.
    pub const fn initial() -> GameContext {
        Self {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00001111,
            captured_piece: PieceType::NoPieceType,
            previous: None,
            zobrist_hash: 0,
            current_side_attacks: AttacksByColor::initial_white(),
            opposite_side_attacks: AttacksByColor::initial_black()
        }
    }

    /// Creates a new context with no previous context.
    pub const fn new_without_previous(side_to_move: Color) -> GameContext {
        GameContext {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0,
            captured_piece: PieceType::NoPieceType,
            previous: None,
            zobrist_hash: 0,
            current_side_attacks: AttacksByColor::new(side_to_move),
            opposite_side_attacks: AttacksByColor::new(side_to_move.other())
        }
    }

    /// Checks if the halfmove clock is valid (less than or equal to 100).
    pub const fn has_valid_halfmove_clock(&self) -> bool {
        self.halfmove_clock <= 100
    }

    /// Gets the last context belonging to a position that could be the same as the current position
    /// (same side to move, nonzero halfmove clock), if it exists.
    /// Else, returns None.
    /// This essentially gets the context of the position two halfmoves ago, if it exists and there
    /// was no halfmove_clock reset in between.
    pub fn get_previous_possible_repetition(&self) -> Option<*mut GameContext> {
        match self.previous {
            Some(previous) => unsafe {
                if (*previous).halfmove_clock == 0 {
                    return None;
                }
                (*previous).previous
            },
            None => None,
        }
    }

    /// Checks if threefold repetition has occurred by checking if the zobrist hash of the current
    /// position has occurred three times, searching backward until the halfmove clock indicates
    /// that no more possible repetitions could have occurred, or until there are no more previous
    /// contexts.
    pub fn has_threefold_repetition_occurred(&self) -> bool {
        if self.halfmove_clock < 4 {
            return false;
        }

        let mut count = 1;

        let mut current_context = self.get_previous_possible_repetition();
        let mut expected_halfmove_clock = self.halfmove_clock - 2;

        unsafe {
            while let Some(context) = current_context {
                if (*context).halfmove_clock != expected_halfmove_clock {
                    break;
                }

                if (*context).zobrist_hash == self.zobrist_hash {
                    count += 1;
                    if count == 3 {
                        return true;
                    }
                }

                expected_halfmove_clock = expected_halfmove_clock.wrapping_sub(2);
                current_context = (*context).get_previous_possible_repetition();
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use crate::{AttacksByColor, Color};
    use crate::PieceType;
    use crate::state::GameContext;

    #[test]
    fn test_initial_context() {
        let context = { 
            let mut context = GameContext::initial();
            context.zobrist_hash = 0x123456789ABCDEF0;
            context
        };

        assert_eq!(context.halfmove_clock, 0);
        assert_eq!(context.double_pawn_push, -1);
        assert_eq!(context.castling_rights, 0b00001111);
        assert_eq!(context.captured_piece, PieceType::NoPieceType);
        assert!(context.previous.is_none());
    }

    #[test]
    fn test_new_without_previous() {
        let context = GameContext::new_without_previous(Color::Black);

        assert_eq!(context.halfmove_clock, 0);
        assert_eq!(context.double_pawn_push, -1);
        assert_eq!(context.captured_piece, PieceType::NoPieceType);
        assert!(context.previous.is_none());
        assert_eq!(context.current_side_attacks, AttacksByColor::new(Color::Black));
        assert_eq!(context.opposite_side_attacks, AttacksByColor::new(Color::White));
    }

    #[test]
    fn test_new_with_previous() {
        let prev_context_current_side_attacks = AttacksByColor::initial_white();
        let prev_context_opposite_side_attacks = AttacksByColor::initial_black();
        let prev_context = {
            let mut context = GameContext::new_without_previous(Color::White);
            context.zobrist_hash = 0x123456789ABCDEF0;
            context.halfmove_clock = 1; // Simulate a previous context with a halfmove clock of 1
            context.castling_rights = 0b00001011; // Simulate castling rights
            context.captured_piece = PieceType::Pawn; // Simulate a captured piece
            context.double_pawn_push = 4; // Simulate a double pawn push
            context.current_side_attacks = prev_context_current_side_attacks.clone();
            context.opposite_side_attacks = prev_context_opposite_side_attacks.clone();
            context
        };

        let new_context =
            unsafe { GameContext::new_with_previous(Box::into_raw(Box::new(prev_context))) };

        assert_eq!(new_context.halfmove_clock, 2); // Incremented from previous
        assert_eq!(new_context.double_pawn_push, -1);
        assert_eq!(new_context.castling_rights, 0b00001011);
        assert_eq!(new_context.captured_piece, PieceType::NoPieceType);
        assert!(new_context.previous.is_some());
        assert_eq!(new_context.zobrist_hash, 0);
        assert_eq!(new_context.current_side_attacks, prev_context_opposite_side_attacks);
        assert_eq!(new_context.opposite_side_attacks, prev_context_current_side_attacks);
    }

    #[test]
    fn test_has_valid_halfmove_clock() {
        // Test valid halfmove clock (0)
        let mut context = GameContext::new_without_previous(Color::Black);
        assert!(context.has_valid_halfmove_clock());

        // Test valid halfmove clock (100)
        context.halfmove_clock = 100;
        assert!(context.has_valid_halfmove_clock());

        // Test invalid halfmove clock (101)
        context.halfmove_clock = 101;
        assert!(!context.has_valid_halfmove_clock());
    }
}
