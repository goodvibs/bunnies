//! KQkq castling rights as a single byte-sized enum (discriminants `0`…`15` = lower four bits).

use super::{color::Color, flank::Flank, square::Square};
use crate::utilities::{Array, IterableEnum, impl_u8_conversions};

/// All 16 combinations of the four castling flags (KQkq). The discriminant equals the **nibble** value
/// used in FEN / Zobrist (`K=8, Q=4, k=2, q=1`).
#[repr(u8)]
#[derive(Clone, Copy, Eq, Debug, Hash)]
#[derive_const(PartialEq, Default)]
pub enum CastlingRights {
    #[default]
    B0000 = 0,
    B0001 = 1,
    B0010 = 2,
    B0011 = 3,
    B0100 = 4,
    B0101 = 5,
    B0110 = 6,
    B0111 = 7,
    B1000 = 8,
    B1001 = 9,
    B1010 = 10,
    B1011 = 11,
    B1100 = 12,
    B1101 = 13,
    B1110 = 14,
    B1111 = 15,
}

impl CastlingRights {
    #[inline]
    pub const fn from_bits(bits: u8) -> Self {
        debug_assert!(bits <= 0b1111);
        unsafe { std::mem::transmute::<u8, CastlingRights>(bits & 0b1111) }
    }

    #[inline]
    pub const fn bits(self) -> u8 {
        self as u8
    }

    #[inline]
    pub const fn has(self, flank: Flank, color: Color) -> bool {
        self.bits() & flank.rights_mask(color) != 0
    }

    #[inline]
    pub const fn intersects(self, mask: u8) -> bool {
        self.bits() & mask != 0
    }

    /// Clears any rights that a move touching `affected_square` invalidates.
    ///
    /// Apply once for the `from` square and once for the `to` square in [`Position::make_move`](crate::types::Position::make_move);
    /// together this handles a king leaving home, a rook leaving its starting corner, and a rook being
    /// captured on its starting corner. All other squares pass through unchanged.
    #[inline]
    pub const fn after_move(self, affected_square: Square) -> Self {
        Self::from_bits(self.bits() & CASTLING_RIGHTS_MASK[affected_square as usize].bits())
    }
}

impl const IterableEnum<16> for CastlingRights {
    const ALL: Array<Self, 16> = Array([
        Self::B0000,
        Self::B0001,
        Self::B0010,
        Self::B0011,
        Self::B0100,
        Self::B0101,
        Self::B0110,
        Self::B0111,
        Self::B1000,
        Self::B1001,
        Self::B1010,
        Self::B1011,
        Self::B1100,
        Self::B1101,
        Self::B1110,
        Self::B1111,
    ]);
}

impl_u8_conversions!(CastlingRights, 16);

/// Per-square AND mask applied to the castling-rights nibble after a move touches that square.
///
/// `make_move` ANDs the previous rights with both `MASK[from]` and `MASK[to]`, which collectively
/// handle every right-clearing case: a king leaving its home, a rook leaving its starting corner,
/// and a rook being captured on its starting corner. All other squares pass through unchanged.
static CASTLING_RIGHTS_MASK: Array<CastlingRights, 64> = Array({
    let mut mask = [CastlingRights::from_bits(0b1111u8); 64];
    mask[Square::E1 as usize] = CastlingRights::from_bits(!0b1100u8 & 0b1111);
    mask[Square::E8 as usize] = CastlingRights::from_bits(!0b0011u8 & 0b1111);
    mask[Square::A1 as usize] = CastlingRights::from_bits(!0b0100u8 & 0b1111);
    mask[Square::H1 as usize] = CastlingRights::from_bits(!0b1000u8 & 0b1111);
    mask[Square::A8 as usize] = CastlingRights::from_bits(!0b0001u8 & 0b1111);
    mask[Square::H8 as usize] = CastlingRights::from_bits(!0b0010u8 & 0b1111);
    mask
});

#[cfg(test)]
mod tests {
    use std::mem::size_of;

    use super::*;

    #[test]
    fn castling_rights_one_byte() {
        assert_eq!(size_of::<CastlingRights>(), 1);
    }
}
