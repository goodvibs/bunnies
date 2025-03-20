use crate::state::State;
use crate::utils::{Color, PieceType};
use crate::utils::masks::{CASTLING_CHECK_MASK_LONG, CASTLING_CHECK_MASK_SHORT, STARTING_KING_ROOK_GAP_LONG, STARTING_KING_ROOK_GAP_SHORT};

impl State {
    /// Returns true if the current side to move can legally castle short.
    /// Else, returns false.
    pub fn can_legally_castle_short(&self, color: Color) -> bool {
        self.has_castling_rights_short(color) && self.has_castling_space_short(color) && self.can_castle_short_without_check(color)
    }

    /// Returns true if the current side to move can legally castle long.
    /// Else, returns false.
    pub fn can_legally_castle_long(&self, color: Color) -> bool {
        self.has_castling_rights_long(color) && self.has_castling_space_long(color) && self.can_castle_long_without_check(color)
    }

    /// Returns whether the current side to move has short castling rights.
    pub fn has_castling_rights_short(&self, color: Color) -> bool {
        self.context.borrow().castling_rights & (0b00001000 >> (color as u8 * 2)) != 0
    }

    /// Returns whether the current side to move has long castling rights.
    pub fn has_castling_rights_long(&self, color: Color) -> bool {
        self.context.borrow().castling_rights & (0b00000100 >> (color as u8 * 2)) != 0
    }

    /// Returns true if the current side to move has no pieces between the king and the rook for short castling.
    /// Else, returns false.
    const fn has_castling_space_short(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_SHORT[color as usize] & self.board.piece_type_masks[PieceType::AllPieceTypes as usize] == 0
    }

    /// Returns true if the current side to move has no pieces between the king and the rook for long castling.
    /// Else, returns false.
    const fn has_castling_space_long(&self, color: Color) -> bool {
        STARTING_KING_ROOK_GAP_LONG[color as usize] & self.board.piece_type_masks[PieceType::AllPieceTypes as usize] == 0
    }

    /// Returns true if the opponent has no pieces that can attack the squares the king moves through for short castling.
    /// Else, returns false.
    fn can_castle_short_without_check(&self, color: Color) -> bool {
        !self.board.is_mask_in_check(CASTLING_CHECK_MASK_SHORT[color as usize], color.flip())
    }

    /// Returns true if the opponent has no pieces that can attack the squares the king moves through for long castling.
    /// Else, returns false.
    fn can_castle_long_without_check(&self, color: Color) -> bool {
        !self.board.is_mask_in_check(CASTLING_CHECK_MASK_LONG[color as usize], color.flip())
    }
}