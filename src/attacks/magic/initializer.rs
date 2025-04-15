use crate::attacks::magic::lookup::MagicAttacksLookup;
use crate::attacks::magic::magic_info::MagicInfo;
use crate::attacks::magic::random::gen_random_magic_number;
use crate::utilities::SquareMasks;
use crate::{Bitboard, BitboardUtils, Square};

/// Struct responsible for initializing the MagicAttacksLookup
pub(crate) struct MagicAttacksInitializer {
    rng: fastrand::Rng,
    min_bits_threshold: u32,
    attacks: Box<[Bitboard]>,
    current_offset: u32,
}

impl MagicAttacksInitializer {
    pub(crate) fn new() -> Self {
        Self {
            rng: fastrand::Rng::new(),
            min_bits_threshold: 6,
            attacks: Box::new([]),
            current_offset: 0,
        }
    }

    pub(crate) fn with_seed(mut self, seed: u64) -> Self {
        self.rng = fastrand::Rng::with_seed(seed);
        self
    }

    pub(crate) fn with_min_bits_threshold(mut self, threshold: u32) -> Self {
        self.min_bits_threshold = threshold;
        self
    }

    /// Initialize the magic attacks lookup object for a sliding piece
    pub(crate) fn init_for_piece(
        &mut self,
        relevant_mask_lookup: &SquareMasks,
        calc_attack_mask: &impl Fn(Square, Bitboard) -> Bitboard,
        table_size: usize,
    ) -> MagicAttacksLookup {
        self.attacks = vec![0; table_size].into_boxed_slice();

        let mut magic_info_for_squares = [MagicInfo::default(); 64];

        for (i, square) in Square::ALL.into_iter().enumerate() {
            magic_info_for_squares[i] = self.generate_magic_info(
                relevant_mask_lookup.get(square),
                |occupied_mask: Bitboard| calc_attack_mask(square, occupied_mask),
            );
        }

        MagicAttacksLookup {
            attacks: std::mem::replace(&mut self.attacks, Box::new([])),
            magic_info_for_squares,
        }
    }

    /// Initialize magic number and attack table for a single square
    fn generate_magic_info(
        &mut self,
        relevant_mask: Bitboard,
        calc_attack_mask: impl Fn(Bitboard) -> Bitboard,
    ) -> MagicInfo {
        let num_relevant_bits = relevant_mask.count_ones() as u8;
        let right_shift_amount = 64 - num_relevant_bits;

        let occupancy_patterns: Vec<Bitboard> = relevant_mask.iter_bit_combinations().collect();
        let attack_masks: Vec<Bitboard> = occupancy_patterns
            .iter()
            .map(|&occupied| calc_attack_mask(occupied))
            .collect();

        let mut attack_table;
        let mut magic_number: Bitboard;

        loop {
            magic_number = gen_random_magic_number(&mut self.rng);

            // Quick rejection test based on bit count heuristic
            if (relevant_mask.wrapping_mul(magic_number) & 0xFF_00_00_00_00_00_00_00).count_ones()
                < self.min_bits_threshold
            {
                continue;
            }

            attack_table = vec![0 as Bitboard; 1 << num_relevant_bits];
            let mut collision = false;

            for (&occupied, &attack_mask) in occupancy_patterns.iter().zip(attack_masks.iter()) {
                let index = ((occupied.wrapping_mul(magic_number)) >> right_shift_amount) as usize;

                if attack_table[index] == 0 {
                    attack_table[index] = attack_mask;
                } else if attack_table[index] != attack_mask {
                    collision = true;
                    break;
                }
            }

            if !collision {
                break;
            }
        }

        let magic_info = MagicInfo {
            relevant_mask,
            magic_number,
            right_shift_amount,
            offset: self.current_offset,
        };

        // Fill the attack table
        for (index, attack_mask) in attack_table.iter().enumerate() {
            if *attack_mask != 0 {
                self.attacks[index + self.current_offset as usize] = *attack_mask;
            }
        }
        self.current_offset += attack_table.len() as u32;
        magic_info
    }
}
