use crate::attacks::magic::lookup::MagicAttacksLookup;
use crate::attacks::magic::magic_info::MagicInfo;
use crate::utilities::{Prng, SquaresToMasks};
use crate::{Bitboard, BitboardUtils, Square};

/// Struct responsible for initializing the MagicAttacksLookup
pub(crate) struct MagicAttacksInitializer {
    rng: Prng,
    attacks: Box<[Bitboard]>,
    current_offset: u32,
}

impl MagicAttacksInitializer {
    pub(crate) fn new(seed: u64, table_size: usize) -> Self {
        Self {
            rng: Prng::new(seed),
            attacks: vec![0; table_size].into_boxed_slice(),
            current_offset: 0,
        }
    }

    /// Initialize the magic attacks lookup object for a sliding piece
    pub(crate) fn init_for_piece(
        &mut self,
        relevant_mask_lookup: SquaresToMasks,
        calc_attack_mask: impl Fn(Square, Bitboard) -> Bitboard,
    ) -> MagicAttacksLookup {
        let mut magic_info_for_squares = [MagicInfo::default(); 64];

        for square in Square::ALL {
            magic_info_for_squares[square as usize] = self.generate_magic_info(
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
            magic_number = self.rng.generate_sparse();

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
