//! Chess ranks 1–8. Line masks: one byte strip per rank, matching [`crate::Square::rank`] (0 = first rank).

use crate::{Bitboard, Color};

/// Algebraic rank: [`Rank::One`] = rank 1 (White’s back rank in the start position), … [`Rank::Eight`] = rank 8.
/// Discriminant matches [`crate::Square::rank`] (`0`…`7`).
#[repr(u8)]
#[derive(Clone, Copy, Eq, Debug, Hash)]
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

    #[inline]
    pub const fn mask(self) -> Bitboard {
        0xFFu64 << ((self as u8) * 8)
    }

    #[inline]
    pub const fn from_u8(rank: u8) -> Self {
        debug_assert!(rank < 8);
        unsafe { std::mem::transmute::<u8, Self>(rank) }
    }

    #[inline]
    pub const fn mirrored(self) -> Self {
        Self::from_u8(7 - self as u8)
    }

    #[inline]
    pub const fn from_perspective(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.mirrored(),
        }
    }
}

impl const PartialEq for Rank {
    fn eq(&self, other: &Self) -> bool {
        *self as u8 == *other as u8
    }
}
