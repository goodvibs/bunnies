//! Precomputed attack tables for non-sliding pieces.

use static_init::dynamic;
use crate::attacks::manual;
use crate::SquareMasks;
use crate::utils::Bitboard;
use crate::utils::Square;

/// Precomputed attacks table for kings.
#[dynamic]
pub static SINGLE_KING_ATTACKS: SquareMasks = SquareMasks::init(&|square| manual::multi_king_attacks(square.mask()));

/// Precomputed attacks table for knights.
#[dynamic]
pub static SINGLE_KNIGHT_ATTACKS: SquareMasks = SquareMasks::init(&|square| manual::multi_knight_attacks(square.mask()));

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
            assert_eq!(precomputed_single_king_attacks(square), manual::multi_king_attacks(square.mask()));
        }
    }

    #[test]
    fn test_single_knight_attacks() {
        for square in Square::ALL {
            assert_eq!(precomputed_single_knight_attacks(square), manual::multi_knight_attacks(square.mask()));
        }
    }
}