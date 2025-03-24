//! Context struct and methods

use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::Bitboard;
use crate::utils::masks::{RANK_6};
use crate::utils::PieceType;

/// A struct containing metadata about the current and past states of the game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct GameContext {
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8, // 0, 0, 0, 0, wk, wq, bk, bq

    // updated after every move
    pub captured_piece: PieceType,
    pub previous: Option<Rc<RefCell<GameContext>>>,
    pub zobrist_hash: Bitboard,
    pub current_side_attacks: Bitboard
}

impl GameContext {
    /// Creates a new context linking to the previous context
    pub fn new_with_previous(previous_context: &Rc<RefCell<GameContext>>, zobrist_hash: Bitboard, current_side_attacks: Bitboard) -> GameContext {
        let (previous_halfmove_clock, previous_castling_rights) = {
            let previous = previous_context.borrow();
            assert_ne!(previous.current_side_attacks, 0, "Previous context must have an updated attacks_mask");
            assert_ne!(previous.zobrist_hash, 0, "Previous context must have an updated zobrist_hash");

            (previous.halfmove_clock, previous.castling_rights)
        };
        GameContext {
            halfmove_clock: previous_halfmove_clock + 1,
            double_pawn_push: -1,
            castling_rights: previous_castling_rights,
            captured_piece: PieceType::NoPieceType,
            previous: Some(Rc::clone(previous_context)),
            zobrist_hash,
            current_side_attacks
        }
    }

    /// Creates a new context with no previous context.
    /// This is used for the initial position.
    pub const fn initial(zobrist_hash: Bitboard) -> GameContext {
        Self::new_without_previous(0b00001111, zobrist_hash, 0xFFFF7E)
    }

    /// Creates a new context with no previous context.
    pub const fn new_without_previous(castling_rights: u8, zobrist_hash: Bitboard, current_side_attacks: Bitboard) -> GameContext {
        GameContext {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights,
            captured_piece: PieceType::NoPieceType,
            previous: None,
            zobrist_hash,
            current_side_attacks
        }
    }

    pub const fn initialize_current_side_attacks(&mut self, attacks_mask: Bitboard) {
        self.current_side_attacks = attacks_mask;
    }

    pub const fn initialize_zobrist_hash(&mut self, zobrist_hash: Bitboard) {
        self.zobrist_hash = zobrist_hash;
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
    pub fn get_previous_possible_repetition(&self) -> Option<Rc<RefCell<GameContext>>> {
        match &self.previous {
            Some(previous) => {
                if previous.borrow().halfmove_clock == 0 {
                    return None;
                }
                match &previous.borrow().previous {
                    Some(previous_previous) => Some(previous_previous.clone()),
                    None => None
                }
            },
            None => None
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
        
        while let Some(context) = current_context {
            let context = context.borrow();
            
            if context.halfmove_clock != expected_halfmove_clock {
                break;
            }
            
            if context.zobrist_hash == self.zobrist_hash {
                count += 1;
                if count == 3 {
                    return true;
                }
            }
            
            expected_halfmove_clock = expected_halfmove_clock.wrapping_sub(2);
            current_context = context.get_previous_possible_repetition();
        }
        
        false
    }
}