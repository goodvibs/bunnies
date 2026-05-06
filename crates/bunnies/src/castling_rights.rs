//! KQkq castling rights as a single byte-sized enum (discriminants `0`…`15` = lower four bits).

use crate::{Color, Flank, Square};

/// All 16 combinations of the four castling flags (KQkq). The discriminant equals the **nibble** value
/// used in FEN / Zobrist (`K=8, Q=4, k=2, q=1`).
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CastlingRights {
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
    pub const NONE: Self = CastlingRights::B0000;
    pub const ALL: Self = CastlingRights::B1111;

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
    /// Apply once for the `from` square and once for the `to` square in [`Position::make_move`];
    /// together this handles a king leaving home, a rook leaving its starting corner, and a rook being
    /// captured on its starting corner. All other squares pass through unchanged.
    #[inline]
    pub const fn after_move(self, affected_square: Square) -> Self {
        Self::from_bits(self.bits() & CASTLING_RIGHTS_MASK[affected_square as usize].bits())
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights::NONE
    }
}

/// Per-square AND mask applied to the castling-rights nibble after a move touches that square.
///
/// `make_move` ANDs the previous rights with both `MASK[from]` and `MASK[to]`, which collectively
/// handle every right-clearing case: a king leaving its home, a rook leaving its starting corner,
/// and a rook being captured on its starting corner. All other squares pass through unchanged.
const CASTLING_RIGHTS_MASK: [CastlingRights; 64] = {
    let mut mask = [CastlingRights::from_bits(0b1111u8); 64];
    mask[Square::E1 as usize] = CastlingRights::from_bits(!0b1100u8 & 0b1111);
    mask[Square::E8 as usize] = CastlingRights::from_bits(!0b0011u8 & 0b1111);
    mask[Square::A1 as usize] = CastlingRights::from_bits(!0b0100u8 & 0b1111);
    mask[Square::H1 as usize] = CastlingRights::from_bits(!0b1000u8 & 0b1111);
    mask[Square::A8 as usize] = CastlingRights::from_bits(!0b0001u8 & 0b1111);
    mask[Square::H8 as usize] = CastlingRights::from_bits(!0b0010u8 & 0b1111);
    mask
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn castling_rights_one_byte() {
        assert_eq!(size_of::<CastlingRights>(), 1);
    }
}
