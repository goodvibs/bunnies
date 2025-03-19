use static_init::dynamic;
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

const RNG_SEED: u64 = 0;

/// Magic dictionaries for rooks
#[dynamic]
pub static ROOK_MAGIC_DICT: MagicDict = MagicDict::init(&ROOK_RELEVANT_MASKS, &manual_single_rook_attacks, ROOK_ATTACK_TABLE_SIZE);

/// Magic dictionaries for bishops
#[dynamic]
pub static BISHOP_MAGIC_DICT: MagicDict = MagicDict::init(&BISHOP_RELEVANT_MASKS, &manual_single_bishop_attacks, BISHOP_ATTACK_TABLE_SIZE);

/// A magic dictionary for a sliding piece
pub struct MagicDict {
    attacks: Box<[Bitboard]>,
    magic_info_for_squares: [MagicInfo; 64],
}

impl MagicDict {
    /// Create a new magic dictionary for a sliding piece
    fn init(relevant_mask_lookup: &PrecomputedMasksForSquares, calc_attack_mask: &impl Fn(Square, Bitboard) -> Bitboard, size: usize) -> Self {
        let mut res = Self {
            attacks: vec![0; size].into_boxed_slice(),
            magic_info_for_squares: [MagicInfo {
                relevant_mask: 0,
                magic_number: 0,
                right_shift_amount: 0,
                offset: 0
            }; 64]
        };
        res.fill_magic_numbers_and_attacks(relevant_mask_lookup, calc_attack_mask);
        res
    }

    /// Get the magic info for a square
    fn get_magic_info_for_square(&self, square: Square) -> MagicInfo {
        self.magic_info_for_squares[square as usize]
    }

    /// Calculate the attack mask for a square with a given occupied mask
    pub fn calc_attack_mask(&self, square: Square, occupied_mask: Bitboard) -> Bitboard {
        let magic_info = self.get_magic_info_for_square(square);
        let key = magic_info.calc_key(occupied_mask);
        self.attacks[key]
    }

    /// Fill the magic numbers and attack tables for all squares
    fn fill_magic_numbers_and_attacks(&mut self, relevant_mask_lookup: &PrecomputedMasksForSquares, calc_attack_mask: &impl Fn(Square, Bitboard) -> Bitboard) {
        let mut current_offset = 0;
        for square in Square::iter_all() {
            self.fill_magic_numbers_and_attacks_for_square(*square, relevant_mask_lookup, calc_attack_mask, &mut current_offset);
        }
    }

    /// Fill the magic numbers and attack tables for a single square
    fn fill_magic_numbers_and_attacks_for_square(&mut self, square: Square, relevant_mask_lookup: &PrecomputedMasksForSquares, calc_attack_mask: &impl Fn(Square, Bitboard) -> Bitboard, current_offset: &mut u32) -> Bitboard {
        let mut rng = fastrand::Rng::with_seed(RNG_SEED);

        let relevant_mask = relevant_mask_lookup.get(square);

        let mut magic_number: Bitboard;

        loop {
            magic_number = gen_random_magic_number(&mut rng);

            // Test if the magic number is suitable based on a quick bit-count heuristic
            if (relevant_mask.wrapping_mul(magic_number) & 0xFF_00_00_00_00_00_00_00).count_ones() < 6 {
                continue;
            }

            let num_relevant_bits = relevant_mask.count_ones() as usize;
            let right_shift_amount = 64 - num_relevant_bits as u8;
            let mut used = vec![0 as Bitboard; 1 << num_relevant_bits];

            let magic_info = MagicInfo { relevant_mask, magic_number, right_shift_amount, offset: *current_offset };

            let mut failed = false;

            for (_i, occupied_mask) in get_bit_combinations_iter(relevant_mask).enumerate() {
                let attack_mask = calc_attack_mask(square, occupied_mask);
                assert_ne!(attack_mask, 0);

                let used_index = magic_info.calc_key_without_offset(occupied_mask);

                // If the index in the used array is not set, store the attack mask
                if used[used_index] == 0 {
                    used[used_index] = attack_mask;
                } else if used[used_index] != attack_mask {
                    // If there's a non-constructive collision, the magic number is not suitable
                    failed = true;
                    break;
                }
            }

            if !failed {
                for (index_without_offset, attack_mask) in used.iter().enumerate() {
                    if *attack_mask == 0 {
                        continue;
                    }
                    self.attacks[index_without_offset + *current_offset as usize] = *attack_mask;
                }
                self.magic_info_for_squares[square as usize] = magic_info;
                *current_offset += used.len() as u32;
                break;
            }
        }

        magic_number
    }
}

#[cfg(test)]
mod tests {
    use crate::attacks::magic::{magic_single_bishop_attacks, magic_single_rook_attacks};
    use crate::attacks::magic::relevant_mask::{BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
    use crate::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};
    use crate::bitboard::get_bit_combinations_iter;
    use crate::charboard::print_bb_pretty;
    use crate::piece_type::PieceType;
    use crate::square::Square;

    #[test]
    fn test_fill_magic_numbers_and_attacks() {
        for sliding_piece in [PieceType::Rook, PieceType::Bishop] {
            for src_square in Square::iter_all() {
                let relevant_mask = match sliding_piece {
                    PieceType::Rook => ROOK_RELEVANT_MASKS.get(*src_square),
                    _ => BISHOP_RELEVANT_MASKS.get(*src_square),
                };
                let occupied_masks_iter = get_bit_combinations_iter(relevant_mask);
                for occupied_mask in occupied_masks_iter {
                    let magic_attacks = match sliding_piece {
                        PieceType::Rook => magic_single_rook_attacks(*src_square, occupied_mask),
                        _ => magic_single_bishop_attacks(*src_square, occupied_mask),
                    };
                    let manual_attacks = match sliding_piece {
                        PieceType::Rook => manual_single_rook_attacks(*src_square, occupied_mask),
                        _ => manual_single_bishop_attacks(*src_square, occupied_mask),
                    };
                    if magic_attacks != manual_attacks {
                        println!("Square mask:");
                        print_bb_pretty(src_square.get_mask());
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