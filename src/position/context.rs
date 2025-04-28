//! Context struct and methods

use crate::Bitboard;
use crate::Piece;

/// A struct containing metadata about the current and past states of the game.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct PositionContext {
    // copied from previous and then possibly modified
    pub halfmove_clock: u8,
    pub double_pawn_push: i8, // file of double pawn push, if any, else -1
    pub castling_rights: u8,  // 0, 0, 0, 0, wk, wq, bk, bq

    // updated after every move
    pub captured_piece: Piece,
    pub previous: Option<*mut PositionContext>,
    pub zobrist_hash: Bitboard,
    pub pinned: Bitboard,
    pub checkers: Bitboard,
}

impl PositionContext {
    /// Creates a new context linking to the previous context
    pub unsafe fn new_with_previous(previous_context: *mut PositionContext) -> PositionContext {
        let (previous_halfmove_clock, previous_castling_rights) = unsafe {
            assert_ne!(
                (*previous_context).zobrist_hash,
                0,
                "Previous context must have an updated zobrist_hash"
            );

            (
                (*previous_context).halfmove_clock,
                (*previous_context).castling_rights,
            )
        };
        PositionContext {
            halfmove_clock: previous_halfmove_clock + 1,
            double_pawn_push: -1,
            castling_rights: previous_castling_rights,
            captured_piece: Piece::Null,
            previous: Some(previous_context),
            zobrist_hash: 0,
            pinned: 0,
            checkers: 0,
        }
    }

    /// Creates a new context with no previous context.
    /// This is used for the initial position.
    pub const fn initial() -> PositionContext {
        Self {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0b00001111,
            captured_piece: Piece::Null,
            previous: None,
            zobrist_hash: 0,
            pinned: 0,
            checkers: 0,
        }
    }

    /// Creates a new context with no previous context.
    pub const fn new_without_previous() -> PositionContext {
        PositionContext {
            halfmove_clock: 0,
            double_pawn_push: -1,
            castling_rights: 0,
            captured_piece: Piece::Null,
            previous: None,
            zobrist_hash: 0,
            pinned: 0,
            checkers: 0,
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
    pub fn get_previous_possible_repetition(&self) -> Option<*mut PositionContext> {
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
