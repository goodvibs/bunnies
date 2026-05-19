//! This module provides functionality for calculating sliding piece attacks using magic bitboards.

use crate::logic::attacks::manual::manual_sliding_piece_attacks;
use crate::types::{Bitboard, BitboardUtils, File, Piece, Rank, Square};
use crate::utilities::{Array, IterableEnum, Prng};
use std::boxed::Box;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::LazyLock;

static ROOK_RELEVANT_MASKS: Array<Bitboard, 64> = Array({
    let mut arr = [0 as Bitboard; 64];
    for square in Square::ALL {
        arr[square as usize] = calc_rook_relevant_mask(square);
    }
    arr
});

static BISHOP_RELEVANT_MASKS: Array<Bitboard, 64> = Array({
    let mut arr = [0 as Bitboard; 64];
    for square in Square::ALL {
        arr[square as usize] = calc_bishop_relevant_mask(square);
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
    square.diagonals_mask()
        & !square.mask()
        & !(File::A.mask() | File::H.mask() | Rank::One.mask() | Rank::Eight.mask())
}

/// Magic info for a single square, using a pointer to its attack subset.
/// This eliminates the need to pass the attacks table during lookup.
#[derive(Copy, Clone)]
pub(crate) struct MagicInfo {
    pub relevant_mask: Bitboard,
    pub magic_number: Bitboard,
    pub right_shift_amount: u8,
    /// Pointer to the start of this square's attacks in the combined table.
    /// The actual attack is accessed at `attacks.as_ptr().add(calc_key(occupied_mask))`.
    /// NonNull is used instead of raw pointer because it is Send + Sync.
    pub attacks: NonNull<Bitboard>,
}

impl Default for MagicInfo {
    fn default() -> Self {
        Self {
            relevant_mask: 0,
            magic_number: 0,
            right_shift_amount: 0,
            // SAFETY: This is a placeholder value; the actual pointer is set during generation.
            // We use NonNull::dangling() which is a valid but non-dereferenceable pointer.
            attacks: NonNull::dangling(),
        }
    }
}

impl MagicInfo {
    /// Calculate the hash key (index) into the attacks table.
    /// Returns the offset from `self.attacks` where the attack bitboard is stored.
    fn calc_key(&self, occupied_mask: Bitboard) -> usize {
        let blockers = occupied_mask & self.relevant_mask;
        let hash = blockers.wrapping_mul(self.magic_number);
        (hash >> self.right_shift_amount) as usize
    }

    /// Get the attack mask for this square with a given occupied mask.
    /// Uses pointer arithmetic for direct access.
    #[inline]
    pub unsafe fn get_attacks(&self, occupied_mask: Bitboard) -> Bitboard {
        let key = self.calc_key(occupied_mask);
        // SAFETY: The attacks pointer was initialized during generation to point
        // within the boxed attacks table. The calc_key result is always within
        // bounds for that square's subset (verified during generation).
        unsafe { *self.attacks.as_ptr().add(key) }
    }

    /// Serialize MagicInfo to bytes (21 bytes total).
    /// Stores offset instead of pointer for portability.
    fn as_bytes(&self, table_base: NonNull<Bitboard>) -> [u8; 21] {
        let mut bytes = [0u8; 21];
        bytes[0..8].copy_from_slice(&self.relevant_mask.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.magic_number.to_le_bytes());
        bytes[16] = self.right_shift_amount;

        // Store offset from table base instead of raw pointer
        let offset = unsafe { self.attacks.as_ptr().offset_from(table_base.as_ptr()) as u32 };
        bytes[17..21].copy_from_slice(&offset.to_le_bytes());

        bytes
    }

    /// Deserialize MagicInfo from bytes, converting offset to pointer.
    fn from_bytes(bytes: &[u8; 21], table_base: NonNull<Bitboard>) -> Self {
        let offset = u32::from_le_bytes(bytes[17..21].try_into().unwrap()) as isize;

        Self {
            relevant_mask: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            magic_number: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            right_shift_amount: bytes[16],
            attacks: unsafe { NonNull::new_unchecked(table_base.as_ptr().offset(offset)) },
        }
    }
}

/// Size of the attack table for rooks.
const ROOK_ATTACK_TABLE_SIZE: usize =
    36 * 2usize.pow(10) + 28 * 2usize.pow(11) + 4 * 2usize.pow(12);

/// Size of the attack table for bishops.
const BISHOP_ATTACK_TABLE_SIZE: usize =
    4 * 2usize.pow(6) + 44 * 2usize.pow(5) + 12 * 2usize.pow(7) + 4 * 2usize.pow(9);

/// Combined size for both rooks and bishops in a single table.
/// Rooks occupy the first portion, bishops follow immediately after.
const COMBINED_TABLE_SIZE: usize = ROOK_ATTACK_TABLE_SIZE + BISHOP_ATTACK_TABLE_SIZE;

/// The bishop magic info starts at this offset in the combined table.
const BISHOP_TABLE_OFFSET: usize = ROOK_ATTACK_TABLE_SIZE;

/// Unified magic attacks lookup for both rooks and bishops.
/// Uses a single combined attacks table to reduce cache pressure and simplify the design.
pub(crate) struct MagicAttacks {
    /// Magic info for all rook squares (indexed by square)
    pub rook_magic_info_lookup: Array<MagicInfo, 64>,
    /// Magic info for all bishop squares (indexed by square)
    pub bishop_magic_info_lookup: Array<MagicInfo, 64>,
    /// Combined attacks table for both pieces.
    /// Rooks: [0..ROOK_ATTACK_TABLE_SIZE)
    /// Bishops: [ROOK_ATTACK_TABLE_SIZE..COMBINED_TABLE_SIZE)
    attacks: Box<[Bitboard; COMBINED_TABLE_SIZE]>,
}

// SAFETY: MagicAttacks contains NonNull pointers that point into its own boxed array.
// The pointers are initialized during construction and never change.
// The boxed array is never moved or reallocated, so the pointers remain valid.
unsafe impl Send for MagicAttacks {}
unsafe impl Sync for MagicAttacks {}

impl MagicAttacks {
    /// Get the attack mask for a rook on a given square with a given occupied mask.
    #[inline]
    pub fn single_rook_attacks(&self, square: Square, occupied_mask: Bitboard) -> Bitboard {
        let magic_info = self.rook_magic_info_lookup[square as usize];
        // SAFETY: The attacks pointer was initialized to point within our boxed table,
        // and the calc_key result is always within bounds for that square's subset.
        unsafe { magic_info.get_attacks(occupied_mask) }
    }

    /// Get the attack mask for a bishop on a given square with a given occupied mask.
    #[inline]
    pub fn single_bishop_attacks(&self, square: Square, occupied_mask: Bitboard) -> Bitboard {
        let magic_info = self.bishop_magic_info_lookup[square as usize];
        // SAFETY: Same as above.
        unsafe { magic_info.get_attacks(occupied_mask) }
    }

    /// Load from file or generate if not present.
    pub fn load_or_generate(
        filename: PathBuf,
        generate: impl FnOnce() -> Self,
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

    /// Save to file in a portable format (offsets, not pointers).
    pub fn save_to_file(&self, filename: &PathBuf) -> io::Result<()> {
        let mut file = fs::File::create(filename)?;
        let table_base = NonNull::new(self.attacks.as_ptr() as *mut Bitboard).unwrap();

        // Write header: number of squares (64) and table size info for validation
        file.write_all(&[64u8])?;
        file.write_all(&(COMBINED_TABLE_SIZE as u64).to_le_bytes())?;

        // Write rook magic info (64 entries)
        for magic_info in &self.rook_magic_info_lookup {
            file.write_all(&magic_info.as_bytes(table_base))?;
        }

        // Write bishop magic info (64 entries)
        for magic_info in &self.bishop_magic_info_lookup {
            file.write_all(&magic_info.as_bytes(table_base))?;
        }

        // Write the combined attack table
        for attack in self.attacks.iter() {
            file.write_all(&attack.to_le_bytes())?;
        }

        Ok(())
    }

    /// Load from file, converting stored offsets back to pointers.
    pub fn load_from_file(filename: &PathBuf) -> io::Result<Self> {
        let mut file = fs::File::open(filename)?;

        // Read and validate header
        let mut header = [0u8; 9];
        file.read_exact(&mut header)?;
        if header[0] != 64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "magic lookup file: expected 64 in header",
            ));
        }
        let stored_table_size = u64::from_le_bytes(header[1..9].try_into().unwrap()) as usize;
        if stored_table_size != COMBINED_TABLE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "magic lookup file: table size mismatch (expected {}, got {})",
                    COMBINED_TABLE_SIZE, stored_table_size
                ),
            ));
        }

        // Allocate the attacks table first (we need its base address for pointer reconstruction)
        let mut attacks = Box::new([0u64; COMBINED_TABLE_SIZE]);
        let table_base = NonNull::new(attacks.as_mut_ptr()).unwrap();

        // Read magic info, converting offsets to pointers
        let mut rook_magic_info = Array([MagicInfo::default(); 64]);
        for square in Square::ALL {
            let mut magic_info_bytes = [0u8; 21];
            file.read_exact(&mut magic_info_bytes)?;
            rook_magic_info[square as usize] = MagicInfo::from_bytes(&magic_info_bytes, table_base);
        }

        let mut bishop_magic_info = Array([MagicInfo::default(); 64]);
        for square in Square::ALL {
            let mut magic_info_bytes = [0u8; 21];
            file.read_exact(&mut magic_info_bytes)?;
            bishop_magic_info[square as usize] =
                MagicInfo::from_bytes(&magic_info_bytes, table_base);
        }

        // Read the attack table
        for attack in attacks.iter_mut() {
            *attack = read_u64(&mut file)?;
        }

        Ok(MagicAttacks {
            rook_magic_info_lookup: rook_magic_info,
            bishop_magic_info_lookup: bishop_magic_info,
            attacks,
        })
    }

    /// Generate both rook and bishop magic tables in a single pass.
    pub fn generate() -> Self {
        let mut attacks = Box::new([0u64; COMBINED_TABLE_SIZE]);

        // Initialize rooks (offset starts at 0)
        let mut rook_initializer =
            PieceMagicInitializer::new(&mut attacks, 0, Prng::new(3141592653589793238));

        let mut rook_magic_info = Array([MagicInfo::default(); 64]);
        for square in Square::ALL {
            rook_magic_info[square as usize] =
                rook_initializer.generate_square_magic::<{ Piece::Rook }>(square);
        }

        // Initialize bishops (offset starts where rooks ended)
        let mut bishop_initializer = PieceMagicInitializer::new(
            &mut attacks,
            BISHOP_TABLE_OFFSET,
            Prng::new(2718281828459045),
        );

        let mut bishop_magic_info = Array([MagicInfo::default(); 64]);
        for square in Square::ALL {
            bishop_magic_info[square as usize] =
                bishop_initializer.generate_square_magic::<{ Piece::Bishop }>(square);
        }

        MagicAttacks {
            rook_magic_info_lookup: rook_magic_info,
            bishop_magic_info_lookup: bishop_magic_info,
            attacks,
        }
    }
}

/// Single lazy-initialized combined magic attacks table.
pub(crate) static MAGIC_ATTACKS: LazyLock<MagicAttacks> = LazyLock::new(|| {
    MagicAttacks::load_or_generate(
        magic_table_path("magic_attacks_lookup.bin"),
        MagicAttacks::generate,
    )
    .expect("magic table load or generate")
});

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

/// DRY magic initializer that handles both pieces using the combined table.
/// Uses a cursor-based approach to sequentially fill the attacks table.
struct PieceMagicInitializer<'a> {
    /// Base pointer to the attacks table (for calculating stored pointers and writing)
    table_base: NonNull<Bitboard>,
    /// Phantom data to hold the lifetime from the mutable borrow of attacks table
    _marker: std::marker::PhantomData<&'a mut [Bitboard; COMBINED_TABLE_SIZE]>,
    /// Current write cursor (offset from table_base)
    current_offset: usize,
    /// Random number generator for finding magic numbers
    rng: Prng,
}

impl<'a> PieceMagicInitializer<'a> {
    fn new(
        attacks: &'a mut [Bitboard; COMBINED_TABLE_SIZE],
        start_offset: usize,
        rng: Prng,
    ) -> Self {
        Self {
            table_base: NonNull::new(attacks.as_mut_ptr()).unwrap(),
            _marker: std::marker::PhantomData,
            current_offset: start_offset,
            rng,
        }
    }

    /// Generate magic info for a single square.
    fn generate_square_magic<const P: Piece>(&mut self, square: Square) -> MagicInfo {
        let relevant_mask = sliding_piece_relevant_mask::<{ P }>(square);
        let num_relevant_bits = relevant_mask.count_ones() as u8;
        let right_shift_amount = 64 - num_relevant_bits;
        let num_blocker_combinations = 1 << num_relevant_bits;

        let mappings =
            self.build_mappings::<{ P }>(square, relevant_mask, num_blocker_combinations);
        let (magic_number, attacks_lookup) =
            self.find_valid_magic_number(right_shift_amount, &mappings);

        // Calculate the pointer to this square's attack subset
        let attacks_ptr = unsafe { self.table_base.add(self.current_offset) };

        // Copy the attacks lookup into the combined table
        unsafe {
            core::ptr::copy_nonoverlapping(
                attacks_lookup.as_ptr(),
                self.table_base.as_ptr().add(self.current_offset),
                num_blocker_combinations,
            );
        }

        self.current_offset += num_blocker_combinations;

        MagicInfo {
            relevant_mask,
            magic_number,
            right_shift_amount,
            attacks: attacks_ptr,
        }
    }

    /// Build mapping from occupancy patterns to attack masks
    fn build_mappings<const P: Piece>(
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
            let magic_number = self.rng.generate() & self.rng.generate() & self.rng.generate();

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
    MAGIC_ATTACKS.single_rook_attacks(src_square, occupied_mask)
}

/// Calculate the attack mask for a bishop on a given square with a given occupied mask
pub fn magic_single_bishop_attacks(src_square: Square, occupied_mask: Bitboard) -> Bitboard {
    MAGIC_ATTACKS.single_bishop_attacks(src_square, occupied_mask)
}

#[cfg(test)]
mod tests {
    use crate::logic::attacks::magic::{
        magic_single_bishop_attacks, magic_single_rook_attacks, sliding_piece_relevant_mask,
    };
    use crate::logic::attacks::manual::{manual_single_bishop_attacks, manual_single_rook_attacks};
    use crate::types::BitboardUtils;
    use crate::types::{Piece, Square};
    use crate::utilities::IterableEnum;

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
