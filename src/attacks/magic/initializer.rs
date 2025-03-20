use crate::attacks::magic::lookup::MagicAttacksLookup;
use crate::attacks::magic::magic_info::MagicInfo;
use crate::attacks::magic::random::gen_random_magic_number;
use crate::attacks::magic::relevant_mask::PrecomputedMasksForSquares;
use crate::bitboard::{get_bit_combinations_iter, Bitboard};
use crate::square::Square;

const DEFAULT_RNG_SEED: u64 = 0;

/// Struct responsible for initializing the MagicAttacksLookup
pub struct MagicAttacksInitializer {
    rng_seed: u64,
    min_bits_threshold: u32,
}

impl MagicAttacksInitializer {
    pub fn new() -> Self {
        Self {
            rng_seed: DEFAULT_RNG_SEED,
            min_bits_threshold: 6,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng_seed = seed;
        self
    }

    pub fn with_min_bits_threshold(mut self, threshold: u32) -> Self {
        self.min_bits_threshold = threshold;
        self
    }

    /// Initialize the magic attacks lookup object for a sliding piece
    pub fn init_for_piece(
        &self,
        relevant_mask_lookup: &PrecomputedMasksForSquares,
        calc_attack_mask: &impl Fn(Square, Bitboard) -> Bitboard,
        table_size: usize
    ) -> MagicAttacksLookup {
        let mut attacks = vec![0; table_size].into_boxed_slice();
        let mut magic_info_for_squares = [MagicInfo {
            relevant_mask: 0,
            magic_number: 0,
            right_shift_amount: 0,
            offset: 0
        }; 64];

        let mut current_offset = 0;
        for square in Square::iter_all() {
            self.init_for_square(
                *square,
                relevant_mask_lookup,
                calc_attack_mask,
                &mut current_offset,
                &mut attacks,
                &mut magic_info_for_squares
            );
        }

        MagicAttacksLookup {
            attacks,
            magic_info_for_squares,
        }
    }

    /// Initialize magic number and attack table for a single square
    fn init_for_square(
        &self,
        square: Square,
        relevant_mask_lookup: &PrecomputedMasksForSquares,
        calc_attack_mask: &impl Fn(Square, Bitboard) -> Bitboard,
        current_offset: &mut u32,
        attacks: &mut Box<[Bitboard]>,
        magic_info_for_squares: &mut [MagicInfo; 64]
    ) {
        let relevant_mask = relevant_mask_lookup.get(square);
        let (magic_number, right_shift_amount, attack_table) =
            self.find_magic_number(square, relevant_mask, calc_attack_mask);

        let magic_info = MagicInfo {
            relevant_mask,
            magic_number,
            right_shift_amount,
            offset: *current_offset
        };

        // Fill the attack table
        for (index, attack_mask) in attack_table.iter().enumerate() {
            if *attack_mask != 0 {
                attacks[index + *current_offset as usize] = *attack_mask;
            }
        }

        magic_info_for_squares[square as usize] = magic_info;
        *current_offset += attack_table.len() as u32;
    }

    /// Find a suitable magic number for a square
    fn find_magic_number(
        &self,
        square: Square,
        relevant_mask: Bitboard,
        calc_attack_mask: &impl Fn(Square, Bitboard) -> Bitboard
    ) -> (Bitboard, u8, Vec<Bitboard>) {
        let mut rng = fastrand::Rng::with_seed(self.rng_seed);
        let num_relevant_bits = relevant_mask.count_ones() as usize;
        let right_shift_amount = 64 - num_relevant_bits as u8;

        // Precompute occupancy patterns and their attack masks
        let occupancy_patterns: Vec<Bitboard> = get_bit_combinations_iter(relevant_mask).collect();
        let attack_masks: Vec<Bitboard> = occupancy_patterns.iter()
            .map(|&occupied| calc_attack_mask(square, occupied))
            .collect();

        loop {
            let magic_number = gen_random_magic_number(&mut rng);

            // Quick rejection test based on bit count heuristic
            if (relevant_mask.wrapping_mul(magic_number) & 0xFF_00_00_00_00_00_00_00)
                .count_ones() < self.min_bits_threshold {
                continue;
            }

            let mut attack_table = vec![0 as Bitboard; 1 << num_relevant_bits];
            let mut collision = false;

            for (i, (&occupied, &attack_mask)) in occupancy_patterns.iter().zip(attack_masks.iter()).enumerate() {
                let index = ((occupied.wrapping_mul(magic_number)) >> right_shift_amount) as usize;

                if attack_table[index] == 0 {
                    attack_table[index] = attack_mask;
                } else if attack_table[index] != attack_mask {
                    collision = true;
                    break;
                }
            }

            if !collision {
                return (magic_number, right_shift_amount, attack_table);
            }
        }
    }
}