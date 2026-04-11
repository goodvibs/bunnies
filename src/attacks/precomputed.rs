//! Precomputed attack tables for non-sliding pieces.

use crate::Bitboard;
use crate::Square;
use crate::attacks::manual;
use crate::utilities::SquaresToMasks;

const SINGLE_KING_ATTACKS_DATA: [Bitboard; 64] = {
    let mut arr = [0u64; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = manual::multi_king_attacks(unsafe { Square::from(i) }.mask());
        i += 1;
    }
    arr
};

const SINGLE_KNIGHT_ATTACKS_DATA: [Bitboard; 64] = {
    let mut arr = [0u64; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = manual::multi_knight_attacks(unsafe { Square::from(i) }.mask());
        i += 1;
    }
    arr
};

/// Precomputed attacks table for kings.
pub static SINGLE_KING_ATTACKS: SquaresToMasks =
    SquaresToMasks::from_array(SINGLE_KING_ATTACKS_DATA);

/// Precomputed attacks table for knights.
pub static SINGLE_KNIGHT_ATTACKS: SquaresToMasks =
    SquaresToMasks::from_array(SINGLE_KNIGHT_ATTACKS_DATA);

/// Returns a precomputed bitboard with all squares attacked by a knight on `src_square`
pub fn precomputed_single_king_attacks(src_square: Square) -> Bitboard {
    SINGLE_KING_ATTACKS.get(src_square)
}

/// Returns a precomputed bitboard with all squares attacked by a knight on `src_square`
pub fn precomputed_single_knight_attacks(src_square: Square) -> Bitboard {
    SINGLE_KNIGHT_ATTACKS.get(src_square)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_king_attacks() {
        for square in Square::ALL {
            assert_eq!(
                precomputed_single_king_attacks(square),
                manual::multi_king_attacks(square.mask())
            );
        }
    }

    #[test]
    fn test_single_knight_attacks() {
        for square in Square::ALL {
            assert_eq!(
                precomputed_single_knight_attacks(square),
                manual::multi_knight_attacks(square.mask())
            );
        }
    }
}
