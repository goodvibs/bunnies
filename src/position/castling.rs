use crate::masks::{STARTING_KING_ROOK_GAP_LONG, STARTING_KING_ROOK_GAP_SHORT};
use crate::position::Position;
use crate::{Color, PieceType, Square};

impl Position {
    /// Returns true if the current side to move can legally castle short.
    /// Else, returns false.
    pub fn can_legally_castle_short(&self) -> bool {
        self.has_castling_rights_short()
            && self.has_castling_space_short()
            && self.can_castle_short_without_check()
    }

    /// Returns true if the current side to move can legally castle long.
    /// Else, returns false.
    pub fn can_legally_castle_long(&self) -> bool {
        self.has_castling_rights_long()
            && self.has_castling_space_long()
            && self.can_castle_long_without_check()
    }

    /// Returns whether the current side to move has short castling rights.
    pub fn has_castling_rights_short(&self) -> bool {
        unsafe {
            (*self.context).castling_rights & (0b00001000 >> (self.side_to_move as u8 * 2)) != 0
        }
    }

    /// Returns whether the current side to move has long castling rights.
    pub fn has_castling_rights_long(&self) -> bool {
        unsafe {
            (*self.context).castling_rights & (0b00000100 >> (self.side_to_move as u8 * 2)) != 0
        }
    }

    /// Returns true if the current side to move has no pieces between the king and the rook for short castling.
    /// Else, returns false.
    const fn has_castling_space_short(&self) -> bool {
        STARTING_KING_ROOK_GAP_SHORT[self.side_to_move as usize]
            & self.board.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize]
            == 0
    }

    /// Returns true if the current side to move has no pieces between the king and the rook for long castling.
    /// Else, returns false.
    const fn has_castling_space_long(&self) -> bool {
        STARTING_KING_ROOK_GAP_LONG[self.side_to_move as usize]
            & self.board.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize]
            == 0
    }

    const fn get_short_castling_gap_square(&self) -> Square {
        match self.side_to_move {
            Color::White => Square::F1,
            Color::Black => Square::F8
        }
    }

    const fn get_long_castling_gap_square(&self) -> Square {
        match self.side_to_move {
            Color::White => Square::D1,
            Color::Black => Square::D8
        }
    }

    /// Returns true if the opponent has no pieces that can attack the square the king moves through for short castling.
    /// Else, returns false.
    fn can_castle_short_without_check(&self) -> bool {
        !self.board.is_square_attacked(self.get_short_castling_gap_square(), self.side_to_move.other())
    }

    /// Returns true if the opponent has no pieces that can attack the square the king moves through for long castling.
    /// Else, returns false.
    fn can_castle_long_without_check(&self) -> bool {
        !self.board.is_square_attacked(self.get_long_castling_gap_square(), self.side_to_move.other())
    }
}
