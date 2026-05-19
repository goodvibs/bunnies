//! Precomputed attack tables for non-sliding pieces.

use crate::logic::attacks::manual;
use crate::types::{Bitboard, Square};
use crate::utilities::{Array, IterableEnum};

static SINGLE_KING_ATTACKS: Array<Bitboard, 64> = Array({
    let mut arr = [0 as Bitboard; 64];
    for square in Square::ALL {
        arr[square as usize] = manual::multi_king_attacks(square.mask());
    }
    arr
});

static SINGLE_KNIGHT_ATTACKS: Array<Bitboard, 64> = Array({
    let mut arr = [0 as Bitboard; 64];
    for square in Square::ALL {
        arr[square as usize] = manual::multi_knight_attacks(square.mask());
    }
    arr
});

/// Returns a precomputed bitboard with all squares attacked by a knight on `src_square`
pub const fn precomputed_single_king_attacks(src_square: Square) -> Bitboard {
    SINGLE_KING_ATTACKS[src_square as usize]
}

/// Returns a precomputed bitboard with all squares attacked by a knight on `src_square`
pub const fn precomputed_single_knight_attacks(src_square: Square) -> Bitboard {
    SINGLE_KNIGHT_ATTACKS[src_square as usize]
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
