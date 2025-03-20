use static_init::dynamic;
use crate::attacks::magic::initializer::MagicAttacksInitializer;
use crate::attacks::magic::magic_info::MagicInfo;
use crate::attacks::magic::random::gen_random_magic_number;
use crate::attacks::magic::relevant_mask::{PrecomputedMasksForSquares, BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
use crate::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};
use crate::bitboard::{get_bit_combinations_iter, Bitboard};
use crate::square::Square;

/// The size of the attack table for rooks
const ROOK_ATTACK_TABLE_SIZE: usize = 36 * 2usize.pow(10) + 28 * 2usize.pow(11) + 4 * 2usize.pow(12);
/// The size of the attack table for bishops
const BISHOP_ATTACK_TABLE_SIZE: usize = 4 * 2usize.pow(6) + 44 * 2usize.pow(5) + 12 * 2usize.pow(7) + 4 * 2usize.pow(9);

#[dynamic]
pub static ROOK_MAGIC_ATTACKS_LOOKUP: MagicAttacksLookup = MagicAttacksInitializer::new()
    .with_seed(0)
    .init_for_piece(&ROOK_RELEVANT_MASKS, &manual_single_rook_attacks, ROOK_ATTACK_TABLE_SIZE);

#[dynamic]
pub static BISHOP_MAGIC_ATTACKS_LOOKUP: MagicAttacksLookup = MagicAttacksInitializer::new()
    .with_seed(0)
    .init_for_piece(&BISHOP_RELEVANT_MASKS, &manual_single_bishop_attacks, BISHOP_ATTACK_TABLE_SIZE);

/// Object that stores all magic-related information for a sliding piece and provides a method to get the attack mask for a given square and occupied mask
pub struct MagicAttacksLookup {
    pub attacks: Box<[Bitboard]>,
    pub magic_info_for_squares: [MagicInfo; 64],
}

impl MagicAttacksLookup {
    /// Get the magic info for a square
    fn get_magic_info_for_square(&self, square: Square) -> MagicInfo {
        self.magic_info_for_squares[square as usize]
    }

    /// Get the attack mask for a square with a given occupied mask
    pub fn get(&self, square: Square, occupied_mask: Bitboard) -> Bitboard {
        let magic_info = self.get_magic_info_for_square(square);
        let key = magic_info.calc_key(occupied_mask);
        self.attacks[key]
    }
}