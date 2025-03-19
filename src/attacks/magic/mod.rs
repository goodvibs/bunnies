use crate::attacks::magic::magic_dict::{BISHOP_MAGIC_DICT, ROOK_MAGIC_DICT};
use crate::bitboard::Bitboard;
use crate::square::Square;

mod magic_dict;
mod random;
mod magic_info;
mod relevant_mask;

/// Calculate the attack mask for a rook on a given square with a given occupied mask
pub fn magic_single_rook_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    ROOK_MAGIC_DICT.calc_attack_mask(src_square, occupied_mask)
}

/// Calculate the attack mask for a bishop on a given square with a given occupied mask
pub fn magic_single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    BISHOP_MAGIC_DICT.calc_attack_mask(src_square, occupied_mask)
}