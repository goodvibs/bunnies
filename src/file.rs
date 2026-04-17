//! Chess files a–h. Line masks derived from a single file-a bitboard, shifted by file index (chmog-style).

use crate::Bitboard;

/// One of eight files (a–h). `A = 0` … `H = 7`, matching [`crate::Square::file`].
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum File {
    A = 0,
    B = 1,
    C = 2,
    D = 3,
    E = 4,
    F = 5,
    G = 6,
    H = 7,
}

impl File {
    pub const ALL: [File; 8] = [
        File::A,
        File::B,
        File::C,
        File::D,
        File::E,
        File::F,
        File::G,
        File::H,
    ];

    /// MSB column in bunnies’ bitboard layout (same as former `FILE_A`).
    const FILE_A: Bitboard = 0x8080_8080_8080_8080;

    /// Full-file bitboard for this file.
    #[inline]
    pub const fn mask(self) -> Bitboard {
        Self::FILE_A >> (self as u8)
    }

    #[inline]
    pub const fn from_u8(file: u8) -> Self {
        assert!(file < 8);
        unsafe { std::mem::transmute::<u8, File>(file) }
    }
}
