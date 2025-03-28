use crate::Bitboard;
use crate::Square;
use crate::attacks::magic::initializer::MagicAttacksInitializer;
use crate::attacks::magic::magic_info::MagicInfo;
use crate::attacks::magic::relevant_mask::{BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
use crate::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};
use static_init::dynamic;
use std::fs::File;
use std::io;
use std::io::{Read, Write};

/// The size of the attack table for rooks
const ROOK_ATTACK_TABLE_SIZE: usize =
    36 * 2usize.pow(10) + 28 * 2usize.pow(11) + 4 * 2usize.pow(12);
/// The size of the attack table for bishops
const BISHOP_ATTACK_TABLE_SIZE: usize =
    4 * 2usize.pow(6) + 44 * 2usize.pow(5) + 12 * 2usize.pow(7) + 4 * 2usize.pow(9);

#[dynamic]
pub static ROOK_MAGIC_ATTACKS_LOOKUP: MagicAttacksLookup =
    MagicAttacksLookup::load_or_generate("data/magic/rook_magic_attacks_lookup.bin", || {
        MagicAttacksInitializer::new()
            .with_seed(3141592653)
            .init_for_piece(
                &ROOK_RELEVANT_MASKS,
                &manual_single_rook_attacks,
                ROOK_ATTACK_TABLE_SIZE,
            )
    })
    .unwrap();

#[dynamic]
pub static BISHOP_MAGIC_ATTACKS_LOOKUP: MagicAttacksLookup =
    MagicAttacksLookup::load_or_generate("data/magic/bishop_magic_attacks_lookup.bin", || {
        MagicAttacksInitializer::new().with_seed(0).init_for_piece(
            &BISHOP_RELEVANT_MASKS,
            &manual_single_bishop_attacks,
            BISHOP_ATTACK_TABLE_SIZE,
        )
    })
    .unwrap();

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

    pub fn load_or_generate(
        filename: &str,
        generate: impl Fn() -> MagicAttacksLookup,
    ) -> io::Result<Self> {
        match MagicAttacksLookup::load_from_file(filename) {
            Ok(lookup) => Ok(lookup),
            Err(_) => {
                let lookup = generate();
                lookup.save_to_file(filename)?;
                Ok(lookup)
            }
        }
    }

    pub fn save_to_file(&self, filename: &str) -> io::Result<()> {
        let mut file = File::create(filename)?;

        // Write the number of squares
        file.write_all(&[64])?;

        // Write magic info for each square
        for magic_info in &self.magic_info_for_squares {
            file.write_all(&magic_info.relevant_mask.to_le_bytes())?;
            file.write_all(&magic_info.magic_number.to_le_bytes())?;
            file.write_all(&[magic_info.right_shift_amount])?;
            file.write_all(&magic_info.offset.to_le_bytes())?;
        }

        // Write the attack table
        for attack in self.attacks.iter() {
            file.write_all(&attack.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn load_from_file(filename: &str) -> io::Result<Self> {
        let mut file = File::open(filename)?;
        let mut buffer = [0u8; 1];

        // Read number of squares (should be 64)
        file.read_exact(&mut buffer)?;
        assert_eq!(buffer[0], 64, "Invalid file format");

        // Read magic info for each square
        let mut magic_info_for_squares = [MagicInfo::default(); 64];
        for square in Square::ALL {
            let mut relevant_mask_bytes = [0u8; 8];
            let mut magic_number_bytes = [0u8; 8];
            let mut right_shift_amount = [0u8; 1];
            let mut offset_bytes = [0u8; 4];

            file.read_exact(&mut relevant_mask_bytes)?;
            file.read_exact(&mut magic_number_bytes)?;
            file.read_exact(&mut right_shift_amount)?;
            file.read_exact(&mut offset_bytes)?;

            magic_info_for_squares[square as usize] = MagicInfo {
                relevant_mask: u64::from_le_bytes(relevant_mask_bytes),
                magic_number: u64::from_le_bytes(magic_number_bytes),
                right_shift_amount: right_shift_amount[0],
                offset: u32::from_le_bytes(offset_bytes),
            };
        }

        // Read the attack table
        let mut attack_table_size = 0;
        for square in Square::ALL {
            let info = &magic_info_for_squares[square as usize];
            let num_bits = info.relevant_mask.count_ones();
            let end_offset = info.offset + (1 << num_bits);
            attack_table_size = attack_table_size.max(end_offset);
        }

        let mut attacks = vec![0u64; attack_table_size as usize].into_boxed_slice();
        for i in 0..attack_table_size as usize {
            let mut attack_bytes = [0u8; 8];
            file.read_exact(&mut attack_bytes)?;
            attacks[i] = u64::from_le_bytes(attack_bytes);
        }

        Ok(MagicAttacksLookup {
            attacks,
            magic_info_for_squares,
        })
    }
}
