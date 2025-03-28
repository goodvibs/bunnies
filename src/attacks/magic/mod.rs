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
    use crate::PieceType;
    use crate::Square;
    use crate::attacks::magic::relevant_mask::{BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
    use crate::attacks::magic::{magic_single_bishop_attacks, magic_single_rook_attacks};
    use crate::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};
    use crate::utilities::iter_bit_combinations;
    use crate::utilities::print_bb_pretty;

    #[test]
    fn test_fill_magic_numbers_and_attacks() {
        for sliding_piece in [PieceType::Rook, PieceType::Bishop] {
            for src_square in Square::ALL {
                let relevant_mask = match sliding_piece {
                    PieceType::Rook => ROOK_RELEVANT_MASKS.get(src_square),
                    _ => BISHOP_RELEVANT_MASKS.get(src_square),
                };
                let occupied_masks_iter = iter_bit_combinations(relevant_mask);
                for occupied_mask in occupied_masks_iter {
                    let magic_attacks = match sliding_piece {
                        PieceType::Rook => magic_single_rook_attacks(src_square, occupied_mask),
                        _ => magic_single_bishop_attacks(src_square, occupied_mask),
                    };
                    let manual_attacks = match sliding_piece {
                        PieceType::Rook => manual_single_rook_attacks(src_square, occupied_mask),
                        _ => manual_single_bishop_attacks(src_square, occupied_mask),
                    };
                    if magic_attacks != manual_attacks {
                        println!("Square mask:");
                        print_bb_pretty(src_square.mask());
                        println!("\nOccupied mask:");
                        print_bb_pretty(occupied_mask);
                        println!("\nMagic attacks:");
                        print_bb_pretty(magic_attacks);
                        println!("\nManual attacks:");
                        print_bb_pretty(manual_attacks);
                    }
                    assert_eq!(magic_attacks, manual_attacks);
                }
            }
        }
    }
}
