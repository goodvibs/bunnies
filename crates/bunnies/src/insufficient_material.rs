use crate::{Board, Color, Piece};

impl Board {
    /// Returns true if there is insufficient material on both sides to checkmate.
    /// This is the case if both sides have any one of the following, and there are no pawns on the board:
    /// A lone king
    /// A king and bishop
    /// A king and knight
    /// A king and two knights, only if the other side is a lone king
    pub fn are_both_sides_insufficient_material(&self, use_uscf_rules: bool) -> bool {
        if self.piece_mask::<{ Piece::Pawn }>()
            | self.piece_mask::<{ Piece::Rook }>()
            | self.piece_mask::<{ Piece::Queen }>()
            != 0
        {
            return false;
        }

        for color_int in Color::White as u8..Color::Black as u8 + 1 {
            let color = Color::from_is_black(color_int != 0);
            let bishops = self.piece_mask::<{ Piece::Bishop }>() & self.color_mask_at(color);
            let num_bishops = bishops.count_ones();
            if num_bishops > 1 {
                return false;
            }

            let knights = self.piece_mask::<{ Piece::Knight }>() & self.color_mask_at(color);
            let num_knights = knights.count_ones();

            if use_uscf_rules && num_knights == 2 && num_bishops == 0 {
                // king and two knights
                let opponent_mask = self.color_mask_at(color.other());
                let all_occupancy_mask = self.piece_mask::<{ Piece::ALL_PIECES }>();
                let opponent_is_lone_king = (opponent_mask & all_occupancy_mask).count_ones() == 1;
                return opponent_is_lone_king;
            }
            if num_knights + num_bishops > 1 {
                return false;
            }
        }

        true
    }
}
