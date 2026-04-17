//! Chess ranks 1–8. Line masks: one byte strip per rank, matching [`crate::Square::rank`] (0 = first rank).

use crate::Bitboard;

/// Algebraic rank: [`Rank::One`] = rank 1 (White’s back rank in the start position), … [`Rank::Eight`] = rank 8.
/// Discriminant matches [`crate::Square::rank`] (`0`…`7`).
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Rank {
    One = 0,
    Two = 1,
    Three = 2,
    Four = 3,
    Five = 4,
    Six = 5,
    Seven = 6,
    Eight = 7,
}

impl Rank {
    pub const ALL: [Rank; 8] = [
        Rank::One,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
    ];

    /// Full-rank bitboard for this rank (same layout as former `RANK_1`…`RANK_8`).
    #[inline]
    pub const fn mask(self) -> Bitboard {
        0xFFu64 << ((self as u8) * 8)
    }

    #[inline]
    pub const fn from_u8(rank: u8) -> Self {
        assert!(rank < 8);
        unsafe { std::mem::transmute::<u8, Rank>(rank) }
    }
}
