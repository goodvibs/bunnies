use crate::position::Board;
use crate::{Color, PieceType};

impl Board {
    /// Returns true if there is insufficient material on both sides to checkmate.
    /// This is the case if both sides have any one of the following, and there are no pawns on the board:
    /// A lone king
    /// A king and bishop
    /// A king and knight
    /// A king and two knights, only if the other side is a lone king
    pub fn are_both_sides_insufficient_material(&self, use_uscf_rules: bool) -> bool {
        if self.piece_type_masks[PieceType::Pawn as usize]
            | self.piece_type_masks[PieceType::Rook as usize]
            | self.piece_type_masks[PieceType::Queen as usize]
            != 0
        {
            return false;
        }

        for color_int in Color::White as u8..Color::Black as u8 + 1 {
            let bishops = self.piece_type_masks[PieceType::Bishop as usize]
                & self.color_masks[color_int as usize];
            let num_bishops = bishops.count_ones();
            if num_bishops > 1 {
                return false;
            }

            let knights = self.piece_type_masks[PieceType::Knight as usize]
                & self.color_masks[color_int as usize];
            let num_knights = knights.count_ones();

            if use_uscf_rules && num_knights == 2 && num_bishops == 0 {
                // king and two knights
                let opposite_side_bb =
                    self.color_masks[Color::from_is_black(color_int != 0).other() as usize];
                let all_occupancy = self.piece_type_masks[PieceType::ALL_PIECE_TYPES as usize];
                let opposite_side_is_lone_king =
                    (opposite_side_bb & all_occupancy).count_ones() == 1;
                return opposite_side_is_lone_king;
            }
            if num_knights + num_bishops > 1 {
                return false;
            }
        }

        true
    }
}
