//! Kingside vs queenside (short vs long castling).

use std::mem;

use crate::Bitboard;
use crate::Color;
use crate::Square;
use crate::file::File;
use crate::rank::Rank;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Flank {
    Kingside = 0,
    Queenside = 1,
}

impl Flank {
    pub const ALL: [Flank; 2] = [Flank::Kingside, Flank::Queenside];

    pub const fn from_bool(is_queenside: bool) -> Self {
        unsafe { mem::transmute(is_queenside) }
    }

    /// Bit mask for this flank in the 4-bit castling rights nibble (same layout as FEN / [`crate::CastlingRights`]).
    pub const fn rights_mask(self, color: Color) -> u8 {
        match self {
            Flank::Kingside => 0b00001000u8 >> (color as u8 * 2),
            Flank::Queenside => 0b00000100u8 >> (color as u8 * 2),
        }
    }

    /// Both castling bits for `color` (white: bits 3–2, black: bits 1–0).
    pub const fn rights_mask_both_flanks(color: Color) -> u8 {
        Flank::Kingside.rights_mask(color) | Flank::Queenside.rights_mask(color)
    }

    /// Kingside: files e–h; queenside: files a–d (former `KING_SIDE` / `QUEEN_SIDE` composites).
    pub const fn half_board_mask(self) -> Bitboard {
        match self {
            Flank::Kingside => File::E.mask() | File::F.mask() | File::G.mask() | File::H.mask(),
            Flank::Queenside => File::A.mask() | File::B.mask() | File::C.mask() | File::D.mask(),
        }
    }

    /// Empty squares required between king and rook in the starting layout (per color and flank).
    pub const fn castling_gap_mask(self, color: Color) -> Bitboard {
        let back = match color {
            Color::White => Rank::One,
            Color::Black => Rank::Eight,
        };
        let gap_files = match self {
            Flank::Kingside => File::F.mask() | File::G.mask(),
            Flank::Queenside => File::B.mask() | File::C.mask() | File::D.mask(),
        };
        back.mask() & gap_files
    }

    /// Squares the king passes through or lands on (excluding start); used for attack tests when castling.
    pub const fn king_path_mask(self, color: Color) -> Bitboard {
        match (color, self) {
            (Color::White, Flank::Kingside) => Square::F1.mask() | Square::G1.mask(),
            (Color::White, Flank::Queenside) => Square::D1.mask() | Square::C1.mask(),
            (Color::Black, Flank::Kingside) => Square::F8.mask() | Square::G8.mask(),
            (Color::Black, Flank::Queenside) => Square::D8.mask() | Square::C8.mask(),
        }
    }

    /// Square the king lands on after castling on this flank.
    pub const fn king_castled_square(self, color: Color) -> Square {
        match (color, self) {
            (Color::White, Flank::Kingside) => Square::G1,
            (Color::White, Flank::Queenside) => Square::C1,
            (Color::Black, Flank::Kingside) => Square::G8,
            (Color::Black, Flank::Queenside) => Square::C8,
        }
    }
}
