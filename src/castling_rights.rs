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
        assert!(bits <= 0b1111);
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

    #[inline]
    pub const fn with_cleared(self, flank: Flank, color: Color) -> Self {
        Self::from_bits(self.bits() & !flank.rights_mask(color))
    }

    #[inline]
    pub const fn with_cleared_color(self, color: Color) -> Self {
        Self::from_bits(self.bits() & !crate::Flank::rights_mask_both_flanks(color))
    }

    /// Clear the right corresponding to a rook on its home corner (FEN loss of castling).
    pub const fn clear_for_rook_square(self, on: Square) -> Self {
        let b = match on {
            Square::A1 => self.bits() & !0b0100,
            Square::H1 => self.bits() & !0b1000,
            Square::A8 => self.bits() & !0b0001,
            Square::H8 => self.bits() & !0b0010,
            _ => self.bits(),
        };
        Self::from_bits(b)
    }
}

impl Default for CastlingRights {
    fn default() -> Self {
        CastlingRights::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn castling_rights_one_byte() {
        assert_eq!(size_of::<CastlingRights>(), 1);
    }
}
