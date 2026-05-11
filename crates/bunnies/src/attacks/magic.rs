//! This module provides functionality for calculating sliding piece attacks using magic bitboards.

use crate::Bitboard;
use crate::BitboardUtils;
use crate::File;
use crate::Piece;
use crate::Rank;
use crate::Square;
use crate::attacks::manual::manual_sliding_piece_attacks;
use crate::square::{DIAGONALS_BL_TO_TR, DIAGONALS_BR_TO_TL};
use crate::utilities::{Array, Prng};
use std::boxed::Box;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::sync::LazyLock;

static ROOK_RELEVANT_MASKS: Array<Bitboard, 64> = Array({
    let mut arr = [0 as Bitboard; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = calc_rook_relevant_mask(Square::from_u8(i));
        i += 1;
    }
    arr
});

static BISHOP_RELEVANT_MASKS: Array<Bitboard, 64> = Array({
    let mut arr = [0 as Bitboard; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = calc_bishop_relevant_mask(Square::from_u8(i));
        i += 1;
    }
    arr
});

pub const fn sliding_piece_relevant_mask<const P: Piece>(square: Square) -> Bitboard {
    match P {
        Piece::Bishop => BISHOP_RELEVANT_MASKS[square as usize],
        Piece::Rook => ROOK_RELEVANT_MASKS[square as usize],
        Piece::Queen => {
            BISHOP_RELEVANT_MASKS[square as usize] | ROOK_RELEVANT_MASKS[square as usize]
        }

        _ => panic!("P is not a sliding piece"),
    }
}

/// Calculate the relevant mask for a rook on a given square
const fn calc_rook_relevant_mask(square: Square) -> Bitboard {
    let file_mask = square.file().mask();
    let rank_mask = square.rank().mask();
    let mut res = (file_mask | rank_mask) & !square.mask();
    const EDGE_MASKS: Array<Bitboard, 4> = Array([
        File::A.mask(),
        File::H.mask(),
        Rank::One.mask(),
        Rank::Eight.mask(),
    ]);
    for edge_mask in EDGE_MASKS {
        if file_mask != edge_mask && rank_mask != edge_mask {
            res &= !edge_mask;
        }
    }
    res
}

/// Calculate the relevant mask for a bishop on a given square
const fn calc_bishop_relevant_mask(square: Square) -> Bitboard {
    let square_mask = square.mask();
    let mut res: Bitboard = 0;
    for diagonal in DIAGONALS_BR_TO_TL {
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
    }
    for diagonal in DIAGONALS_BL_TO_TR {
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
    }
    res & !square_mask & !(File::A.mask() | File::H.mask() | Rank::One.mask() | Rank::Eight.mask())
}

/// Struct to store all magic-related information for a square
#[derive(Copy, Clone)]
#[derive_const(Default)]
pub(crate) struct MagicInfo {
    pub relevant_mask: Bitboard,
    pub magic_number: Bitboard,
    pub right_shift_amount: u8,
    pub offset: u32,
}

impl MagicInfo {
    fn calc_key(&self, occupied_mask: Bitboard) -> usize {
        let blockers = occupied_mask & self.relevant_mask;
        let mut hash = blockers.wrapping_mul(self.magic_number);
        hash >>= self.right_shift_amount;
        hash as usize + self.offset as usize
    }

    /// Serialize MagicInfo to bytes (21 bytes total)
    fn to_bytes(&self) -> [u8; 21] {
        let mut bytes = [0u8; 21];
        bytes[0..8].copy_from_slice(&self.relevant_mask.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.magic_number.to_le_bytes());
        bytes[16] = self.right_shift_amount;
        bytes[17..21].copy_from_slice(&self.offset.to_le_bytes());
        bytes
    }

    /// Deserialize MagicInfo from bytes
    fn from_bytes(bytes: &[u8; 21]) -> Self {
        Self {
            relevant_mask: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            magic_number: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            right_shift_amount: bytes[16],
            offset: u32::from_le_bytes(bytes[17..21].try_into().unwrap()),
        }
    }
}

/// The size of the attack table for rooks
const ROOK_ATTACK_TABLE_SIZE: usize =
    36 * 2usize.pow(10) + 28 * 2usize.pow(11) + 4 * 2usize.pow(12);
/// The size of the attack table for bishops
const BISHOP_ATTACK_TABLE_SIZE: usize =
    4 * 2usize.pow(6) + 44 * 2usize.pow(5) + 12 * 2usize.pow(7) + 4 * 2usize.pow(9);

pub(crate) static ROOK_MAGIC_ATTACKS_LOOKUP: LazyLock<MagicAttacksLookup<ROOK_ATTACK_TABLE_SIZE>> =
    LazyLock::new(|| {
        MagicAttacksLookup::load_or_generate(
            magic_table_path("rook_magic_attacks_lookup.bin"),
            || MagicInitializer::<ROOK_ATTACK_TABLE_SIZE, { Piece::Rook }>::generate(3141592653),
        )
        .expect("rook magic table load or generate")
    });

pub(crate) static BISHOP_MAGIC_ATTACKS_LOOKUP: LazyLock<
    MagicAttacksLookup<BISHOP_ATTACK_TABLE_SIZE>,
> = LazyLock::new(|| {
    MagicAttacksLookup::load_or_generate(
        magic_table_path("bishop_magic_attacks_lookup.bin"),
        || MagicInitializer::<BISHOP_ATTACK_TABLE_SIZE, { Piece::Bishop }>::generate(0),
    )
    .expect("bishop magic table load or generate")
});

/// Object that stores all magic-related information for a sliding piece and provides a method to get the attack mask for a given square and occupied mask
pub(crate) struct MagicAttacksLookup<const TS: usize> {
    pub attacks: Box<[Bitboard; TS]>,
    pub magic_info_for_squares: Array<MagicInfo, 64>,
}

impl<const TS: usize> MagicAttacksLookup<TS> {
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
        filename: PathBuf,
        generate: impl Fn() -> MagicAttacksLookup<TS>,
    ) -> io::Result<Self> {
        match Self::load_from_file(&filename) {
            Ok(lookup) => Ok(lookup),
            Err(_) => {
                let lookup = generate();
                lookup.save_to_file(&filename)?;
                Ok(lookup)
            }
        }
    }

    pub fn save_to_file(&self, filename: &PathBuf) -> io::Result<()> {
        let mut file = fs::File::create(filename)?;

        // Write the number of squares
        file.write_all(&[64])?;

        // Write magic info for each square
        for magic_info in &self.magic_info_for_squares {
            file.write_all(&magic_info.to_bytes())?;
        }

        // Write the attack table
        for attack in self.attacks.iter() {
            file.write_all(&attack.to_le_bytes())?;
        }

        Ok(())
    }

    pub fn load_from_file(filename: &PathBuf) -> io::Result<Self> {
        let mut file = fs::File::open(filename)?;
        let mut buffer = [0u8; 1];

        // Read number of squares (should be 64)
        file.read_exact(&mut buffer)?;
        if buffer[0] != 64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "magic lookup file: expected 64 in header",
            ));
        }

        // Read magic info for each square
        let mut magic_info_for_squares = Array([MagicInfo::default(); 64]);
        for square in Square::ALL {
            let mut magic_info_bytes = [0u8; 21];
            file.read_exact(&mut magic_info_bytes)?;
            magic_info_for_squares[square as usize] = MagicInfo::from_bytes(&magic_info_bytes);
        }

        // Read the attack table into a heap-allocated array
        let mut attacks = Box::new([0u64; TS]);
        for attack in attacks.iter_mut() {
            *attack = read_u64(&mut file)?;
        }

        Ok(MagicAttacksLookup {
            attacks,
            magic_info_for_squares,
        })
    }
}

fn magic_table_path(file_name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../data/magic")
        .join(file_name)
}

/// Read a u64 from a file in little-endian format
fn read_u64(file: &mut fs::File) -> io::Result<u64> {
    let mut bytes = [0u8; 8];
    file.read_exact(&mut bytes)?;
    Ok(u64::from_le_bytes(bytes))
}

/// Struct responsible for initializing the MagicAttacksLookup
pub(crate) struct MagicInitializer<const TS: usize, const P: Piece> {
    rng: Prng,
    attacks: Box<[Bitboard; TS]>,
    current_offset: u32,
}

impl<const TS: usize, const P: Piece> MagicInitializer<TS, P> {
    /// Generate the magic attacks lookup table for a sliding piece
    pub(crate) fn generate(seed: u64) -> MagicAttacksLookup<TS> {
        let mut initializer = Self {
            rng: Prng::new(seed),
            attacks: Box::new([0; TS]),
            current_offset: 0,
        };

        let mut magic_info_for_squares = Array([MagicInfo::default(); 64]);

        for square in Square::ALL {
            magic_info_for_squares[square as usize] = initializer.generate_magic_info(square);
        }

        MagicAttacksLookup {
            attacks: initializer.attacks,
            magic_info_for_squares,
        }
    }

    /// Initialize magic number and attack table for a single square
    fn generate_magic_info(&mut self, from: Square) -> MagicInfo {
        let relevant_mask = sliding_piece_relevant_mask::<{ P }>(from);
        let num_relevant_bits = relevant_mask.count_ones() as u8;
        let right_shift_amount = 64 - num_relevant_bits;
        let num_blocker_combinations = 1 << num_relevant_bits;

        let mappings = self.build_mappings(from, relevant_mask, num_blocker_combinations);
        let (magic_number, attacks_lookup) =
            self.find_valid_magic_number(right_shift_amount, &mappings);

        let magic_info = MagicInfo {
            relevant_mask,
            magic_number,
            right_shift_amount,
            offset: self.current_offset,
        };

        unsafe {
            core::ptr::copy_nonoverlapping(
                attacks_lookup.as_ptr(),
                self.attacks.as_mut_ptr().add(self.current_offset as usize),
                num_blocker_combinations,
            );
        }

        self.current_offset += num_blocker_combinations as u32;
        magic_info
    }

    /// Build mapping from occupancy patterns to attack masks
    fn build_mappings(
        &self,
        from: Square,
        relevant_mask: Bitboard,
        num_mappings: usize,
    ) -> Vec<(Bitboard, Bitboard)> {
        let mut mappings = Vec::with_capacity(num_mappings);
        for occupancy_pattern in relevant_mask.iter_bit_combinations() {
            mappings.push((
                occupancy_pattern,
                manual_sliding_piece_attacks::<{ P }>(from, occupancy_pattern),
            ));
        }
        mappings
    }

    /// Find a magic number without collisions, returning the magic number and attack lookup table
    fn find_valid_magic_number(
        &mut self,
        right_shift_amount: u8,
        mappings: &[(Bitboard, Bitboard)],
    ) -> (Bitboard, Vec<Bitboard>) {
        loop {
            let magic_number = self.rng.generate_sparse();

            match Self::test_magic_number(magic_number, right_shift_amount, mappings) {
                Some(attacks_lookup) => return (magic_number, attacks_lookup),
                None => continue,
            }
        }
    }

    /// Test if a magic number has collisions. Returns Some(attacks_lookup) if valid, None if collision detected.
    fn test_magic_number(
        magic_number: Bitboard,
        right_shift_amount: u8,
        mappings: &[(Bitboard, Bitboard)],
    ) -> Option<Vec<Bitboard>> {
        let mut attacks_lookup = vec![0 as Bitboard; mappings.len()];

        for (blocker_pattern, attacks) in mappings {
            let index =
                ((blocker_pattern.wrapping_mul(magic_number)) >> right_shift_amount) as usize;

            if attacks_lookup[index] == 0 {
                attacks_lookup[index] = *attacks;
            } else if attacks_lookup[index] != *attacks {
                return None; // Collision detected
            }
        }

        Some(attacks_lookup)
    }
}

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
    use crate::BitboardUtils;
    use crate::attacks::magic::{
        magic_single_bishop_attacks, magic_single_rook_attacks, sliding_piece_relevant_mask,
    };
    use crate::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};
    use crate::{Piece, Square};

    fn assert_magic_matches_manual<const P: Piece>(
        manual_attacks_for: impl Fn(Square, u64) -> u64,
        magic_attacks_for: impl Fn(Square, u64) -> u64,
    ) {
        for src_square in Square::ALL {
            let relevant_mask = sliding_piece_relevant_mask::<{ P }>(src_square);
            for occupied_mask in relevant_mask.iter_bit_combinations() {
                assert_eq!(
                    magic_attacks_for(src_square, occupied_mask),
                    manual_attacks_for(src_square, occupied_mask)
                );
            }
        }
    }

    #[test]
    fn test_fill_magic_numbers_and_attacks() {
        assert_magic_matches_manual::<{ Piece::Rook }>(
            manual_single_rook_attacks,
            magic_single_rook_attacks,
        );
        assert_magic_matches_manual::<{ Piece::Bishop }>(
            manual_single_bishop_attacks,
            magic_single_bishop_attacks,
        );
    }
}
