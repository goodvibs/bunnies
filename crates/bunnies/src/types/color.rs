//! Piece color and side-to-move marker.

use super::{rank::Rank, square::Square};
use crate::utilities::{Array, IterableEnum, impl_u8_conversions};

/// Chess color (White/Black) used for pieces and as a const-generic side-to-move marker.
///
/// Used extensively as a const generic `const STM: Color` on [`Position<N, STM>`](crate::types::Position)
/// to encode the side to move at compile time, enabling zero-cost type-state assertions.
#[repr(u8)]
#[derive(Clone, Copy, Eq, Debug, std::marker::ConstParamTy)]
#[derive_const(PartialEq)]
pub enum Color {
    /// White pieces / White to move.
    White = 0,
    /// Black pieces / Black to move.
    Black = 1,
}

impl Color {
    /// Converts from a boolean: `false` → White, `true` → Black.
    pub const fn from_is_black(is_black: bool) -> Color {
        unsafe { std::mem::transmute::<bool, Color>(is_black) }
    }

    /// The opposite color (White ↔ Black).
    ///
    /// Takes `self` by value so this method can be used in const-generic position:
    /// `Position<N, { STM.other() }>`.
    pub const fn other(self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    /// Initial king square for this color (E1 for White, E8 for Black).
    pub const fn king_initial_square(self: Color) -> Square {
        match self {
            Color::White => Square::E1,
            Color::Black => Square::E8,
        }
    }

    /// The rank where pawns of this color can capture en passant.
    pub const fn en_passant_capture_rank(self) -> Rank {
        match self {
            Self::White => Rank::Five,
            Self::Black => Rank::Four,
        }
    }
}

impl const IterableEnum<2> for Color {
    const ALL: Array<Color, 2> = Array([Color::White, Color::Black]);
}

impl_u8_conversions!(Color, 2);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        assert_eq!(Color::White as u8, 0);
        assert_eq!(Color::Black as u8, 1);
        assert_eq!(Color::White.other(), Color::Black);
        assert_eq!(Color::Black.other(), Color::White);
        assert_eq!(Color::from_is_black(false), Color::White);
        assert_eq!(Color::from_is_black(true), Color::Black);
    }
}
