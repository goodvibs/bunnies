//! This module provides functionality for calculating sliding piece attacks using magic bitboards.

use crate::Bitboard;
use crate::Square;
use crate::attacks::magic::lookup::{BISHOP_MAGIC_ATTACKS_LOOKUP, ROOK_MAGIC_ATTACKS_LOOKUP};

mod initializer;
mod lookup;
mod magic_info;
mod random;
mod relevant_mask;

pub use relevant_mask::*;

/// Calculate the attack mask for a rook on a given square with a given occupied mask
pub fn magic_single_rook_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    ROOK_MAGIC_ATTACKS_LOOKUP.get(src_square, occupied_mask)
}

/// Calculate the attack mask for a bishop on a given square with a given occupied mask
pub fn magic_single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    BISHOP_MAGIC_ATTACKS_LOOKUP.get(src_square, occupied_mask)
}

#[cfg(test)]
mod tests {
    use crate::Square;
    use crate::attacks::magic::relevant_mask::{BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
    use crate::attacks::magic::{magic_single_bishop_attacks, magic_single_rook_attacks};
    use crate::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};
    use crate::{BitboardUtils, Piece};

    #[test]
    fn test_fill_magic_numbers_and_attacks() {
        for sliding_piece in [Piece::Rook, Piece::Bishop] {
            for src_square in Square::ALL {
                let relevant_mask = match sliding_piece {
                    Piece::Rook => ROOK_RELEVANT_MASKS.get(src_square),
                    _ => BISHOP_RELEVANT_MASKS.get(src_square),
                };
                let occupied_masks_iter = relevant_mask.iter_bit_combinations();
                for occupied_mask in occupied_masks_iter {
                    let magic_attacks = match sliding_piece {
                        Piece::Rook => magic_single_rook_attacks(src_square, occupied_mask),
                        _ => magic_single_bishop_attacks(src_square, occupied_mask),
                    };
                    let manual_attacks = match sliding_piece {
                        Piece::Rook => manual_single_rook_attacks(src_square, occupied_mask),
                        _ => manual_single_bishop_attacks(src_square, occupied_mask),
                    };
                    assert_eq!(magic_attacks, manual_attacks);
                }
            }
        }
    }
}
