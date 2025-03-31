use crate::{Bitboard, Board, Color};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttacksByColor {
    pub all: Bitboard,
    pub non_sliding: Bitboard,
    pub bishops: Bitboard,
    pub rooks: Bitboard,
    pub queens: Bitboard,
    pub side: Color,
}

impl AttacksByColor {
    pub const fn new(side: Color) -> Self {
        Self {
            all: 0,
            non_sliding: 0,
            bishops: 0,
            rooks: 0,
            queens: 0,
            side,
        }
    }
    
    pub const fn initial_white() -> Self {
        Self {
            all: 0xFF_FF_7E,
            non_sliding: 0xFF_1C_14,
            rooks: 0x81_42,
            bishops: 0x5A_00,
            queens: 0x38_28,
            side: Color::White,
        }
    }
    
    pub const fn initial_black() -> Self {
        Self {
            all: 0x7E_FF_FF_00_00_00_00_00,
            non_sliding: 0x14_1C_FF_00_00_00_00_00,
            rooks: 0x42_81_00_00_00_00_00_00,
            bishops: 0x00_5A_00_00_00_00_00_00,
            queens: 0x28_38_00_00_00_00_00_00,
            side: Color::Black,
        }
    }
    
    pub fn update(&mut self, board: &Board) {
        self.non_sliding = board.calc_non_sliding_piece_attacks_mask(self.side);
        self.bishops = board.calc_bishop_attacks_mask(self.side);
        self.rooks = board.calc_rook_attacks_mask(self.side);
        self.queens = board.calc_queen_attacks_mask(self.side);

        self.all = self.non_sliding | self.bishops | self.rooks | self.queens;
    }
}