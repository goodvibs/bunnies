//! Kingside vs queenside (short vs long castling).

use crate::Bitboard;
use crate::Color;
use crate::Square;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Flank {
    Kingside = 0,
    Queenside = 1,
}

impl Flank {
    pub const ALL: [Flank; 2] = [Flank::Kingside, Flank::Queenside];

    /// Bit mask for this flank in the 4-bit castling rights nibble (same layout as FEN / `PositionContext::castling_rights`).
    pub const fn rights_mask(self, color: Color) -> u8 {
        match self {
            Flank::Kingside => 0b00001000u8 >> (color as u8 * 2),
            Flank::Queenside => 0b00000100u8 >> (color as u8 * 2),
        }
    }

    /// Squares the king passes through or lands on (excluding start); used for attack tests when castling.
    pub fn king_path_mask(self, color: Color) -> Bitboard {
        match (color, self) {
            (Color::White, Flank::Kingside) => Square::F1.mask() | Square::G1.mask(),
            (Color::White, Flank::Queenside) => Square::D1.mask() | Square::C1.mask(),
            (Color::Black, Flank::Kingside) => Square::F8.mask() | Square::G8.mask(),
            (Color::Black, Flank::Queenside) => Square::D8.mask() | Square::C8.mask(),
        }
    }
}
