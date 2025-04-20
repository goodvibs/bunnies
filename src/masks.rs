//! This module contains bitboard masks that may be useful.

use crate::Bitboard;

pub const FILE_A: Bitboard = 0x8080808080808080;
pub const FILE_B: Bitboard = 0x4040404040404040;
pub const FILE_C: Bitboard = 0x2020202020202020;
pub const FILE_D: Bitboard = 0x1010101010101010;
pub const FILE_E: Bitboard = 0x0808080808080808;
pub const FILE_F: Bitboard = 0x0404040404040404;
pub const FILE_G: Bitboard = 0x0202020202020202;
pub const FILE_H: Bitboard = 0x0101010101010101;
pub const FILES_AB: Bitboard = FILE_A | FILE_B;
pub const FILES_GH: Bitboard = FILE_G | FILE_H;

pub const RANK_1: Bitboard = 0x00000000000000FF;
pub const RANK_2: Bitboard = 0x000000000000FF00;
pub const RANK_3: Bitboard = 0x0000000000FF0000;
pub const RANK_4: Bitboard = 0x00000000FF000000;
pub const RANK_5: Bitboard = 0x000000FF00000000;
pub const RANK_6: Bitboard = 0x0000FF0000000000;
pub const RANK_7: Bitboard = 0x00FF000000000000;
pub const RANK_8: Bitboard = 0xFF00000000000000;

pub const KING_SIDE: Bitboard = FILE_E | FILE_F | FILE_G | FILE_H;
pub const QUEEN_SIDE: Bitboard = FILE_A | FILE_B | FILE_C | FILE_D;
pub const OUTER_EDGES: Bitboard = FILE_A | FILE_H | RANK_1 | RANK_8;

pub const STARTING_WK_WR_GAP_SHORT: Bitboard = RANK_1 & (FILE_F | FILE_G);
pub const STARTING_WK_WR_GAP_LONG: Bitboard = RANK_1 & (FILE_B | FILE_C | FILE_D);
pub const STARTING_BK_BR_GAP_SHORT: Bitboard = RANK_8 & (FILE_F | FILE_G);
pub const STARTING_BK_BR_GAP_LONG: Bitboard = RANK_8 & (FILE_B | FILE_C | FILE_D);

pub const STARTING_KING_ROOK_GAP_SHORT: [Bitboard; 2] =
    [STARTING_WK_WR_GAP_SHORT, STARTING_BK_BR_GAP_SHORT];
pub const STARTING_KING_ROOK_GAP_LONG: [Bitboard; 2] =
    [STARTING_WK_WR_GAP_LONG, STARTING_BK_BR_GAP_LONG];

pub const FILES: [Bitboard; 8] = [
    FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
];

pub const RANKS: [Bitboard; 8] = [
    RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
];

/// Diagonal masks starting from the bottom right corner to the top left corner.
pub const DIAGONALS_BR_TO_TL: [Bitboard; 15] = [
    0x0000000000000001,
    0x0000000000000102,
    0x0000000000010204,
    0x0000000001020408,
    0x0000000102040810,
    0x0000010204081020,
    0x0001020408102040,
    0x0102040810204080,
    0x0204081020408000,
    0x0408102040800000,
    0x0810204080000000,
    0x1020408000000000,
    0x2040800000000000,
    0x4080000000000000,
    0x8000000000000000,
];

/// Diagonal masks starting from the bottom left corner to the top right corner.
pub const DIAGONALS_BL_TO_TR: [Bitboard; 15] = [
    0x0000000000000080,
    0x0000000000008040,
    0x0000000000804020,
    0x0000000080402010,
    0x0000008040201008,
    0x0000804020100804,
    0x0080402010080402,
    0x8040201008040201,
    0x4020100804020100,
    0x2010080402010000,
    0x1008040201000000,
    0x0804020100000000,
    0x0402010000000000,
    0x0201000000000000,
    0x0100000000000000,
];

pub const STARTING_WP: Bitboard = RANK_2;
pub const STARTING_WN: Bitboard = 0x0000000000000042;
pub const STARTING_WB: Bitboard = 0x0000000000000024;
pub const STARTING_WR: Bitboard = 0x0000000000000081;
pub const STARTING_WQ: Bitboard = 0x0000000000000010;
pub const STARTING_WK: Bitboard = 0x0000000000000008;
pub const STARTING_BP: Bitboard = RANK_7;
pub const STARTING_BN: Bitboard = 0x4200000000000000;
pub const STARTING_BB: Bitboard = 0x2400000000000000;
pub const STARTING_BR: Bitboard = 0x8100000000000000;
pub const STARTING_BQ: Bitboard = 0x1000000000000000;
pub const STARTING_BK: Bitboard = 0x0800000000000000;

pub const STARTING_WHITE: Bitboard =
    STARTING_WP | STARTING_WN | STARTING_WB | STARTING_WR | STARTING_WQ | STARTING_WK;
pub const STARTING_BLACK: Bitboard =
    STARTING_BP | STARTING_BN | STARTING_BB | STARTING_BR | STARTING_BQ | STARTING_BK;
pub const STARTING_ALL: Bitboard = STARTING_WHITE | STARTING_BLACK;

pub const STARTING_KING_SIDE_WR: Bitboard = 0x0000000000000001;
pub const STARTING_QUEEN_SIDE_WR: Bitboard = 0x0000000000000080;
pub const STARTING_KING_SIDE_BR: Bitboard = 0x0100000000000000;
pub const STARTING_QUEEN_SIDE_BR: Bitboard = 0x8000000000000000;

pub const STARTING_KING_SIDE_ROOK: [Bitboard; 2] = [STARTING_KING_SIDE_WR, STARTING_KING_SIDE_BR];
pub const STARTING_QUEEN_SIDE_ROOK: [Bitboard; 2] =
    [STARTING_QUEEN_SIDE_WR, STARTING_QUEEN_SIDE_BR];


#[cfg(test)]
mod tests {
    use crate::masks::{DIAGONALS_BL_TO_TR, DIAGONALS_BR_TO_TL, FILES, FILE_A, RANKS, RANK_1};
    use crate::Square;

    #[test]
    fn test_files() {
        assert_eq!(FILES.len(), 8);
        assert_eq!(FILE_A, Square::A1.mask() | Square::A2.mask() | Square::A3.mask() | Square::A4.mask() | Square::A5.mask() | Square::A6.mask() | Square::A7.mask() | Square::A8.mask());
        
        let mut expected_file = FILE_A;
        for file in FILES {
            assert_eq!(file, expected_file);
            expected_file >>= 1;
        }
    }
    
    #[test]
    fn test_ranks() {
        assert_eq!(RANKS.len(), 8);
        assert_eq!(RANK_1, Square::A1.mask() | Square::B1.mask() | Square::C1.mask() | Square::D1.mask() | Square::E1.mask() | Square::F1.mask() | Square::G1.mask() | Square::H1.mask());
        
        let mut expected_rank = RANK_1;
        for rank in RANKS {
            assert_eq!(rank, expected_rank);
            expected_rank <<= 8;
        }
    }

    #[test]
    fn test_diagonals_br_to_tl() {
        assert_eq!(DIAGONALS_BR_TO_TL.len(), 15);
        assert_eq!(DIAGONALS_BR_TO_TL[0], 0x1);
        
        let mut mask = 0;
        for diagonal in DIAGONALS_BR_TO_TL {
            assert_ne!(diagonal, 0);
            assert_eq!(mask ^ diagonal, mask | diagonal);
            mask |= diagonal;
        }
        assert_eq!(mask.count_zeros(), 0);
    }
    
    #[test]
    fn test_diagonals_bl_to_tr() {
        assert_eq!(DIAGONALS_BL_TO_TR.len(), 15);
        assert_eq!(DIAGONALS_BL_TO_TR[0], 0x80);
        
        let mut mask = 0;
        for diagonal in DIAGONALS_BL_TO_TR {
            assert_ne!(diagonal, 0);
            assert_eq!(mask ^ diagonal, mask | diagonal);
            mask |= diagonal;
        }
        assert_eq!(mask.count_zeros(), 0);
    }
}