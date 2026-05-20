//! Chess ranks 1–8. Line masks: one byte strip per rank, matching [`crate::Square::rank`] (0 = first rank).

use super::bitboard::Bitboard;
use super::color::Color;
use crate::{
    utilities::impl_u8_conversions,
    utilities::{Array, IterableEnum},
};

/// Algebraic rank: [`Rank::One`] = rank 1 (White’s back rank in the start position), … [`Rank::Eight`] = rank 8.
/// Discriminant matches [`crate::Square::rank`] (`0`…`7`).
#[repr(u8)]
#[derive(Clone, Copy, Eq, Debug)]
#[derive_const(PartialEq)]
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
    #[inline]
    pub const fn mask(self) -> Bitboard {
        0xFFu64 << ((self as u8) * 8)
    }

    #[inline]
    pub const fn mirrored(self) -> Self {
        unsafe { (7 - self as u8).try_into().unwrap_unchecked() }
    }

    #[inline]
    pub const fn from_perspective(self, color: Color) -> Self {
        match color {
            Color::White => self,
            Color::Black => self.mirrored(),
        }
    }
}

impl const IterableEnum<8> for Rank {
    const ALL: Array<Rank, 8> = Array([
        Rank::One,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
    ]);
}

impl_u8_conversions!(Rank, 8);
