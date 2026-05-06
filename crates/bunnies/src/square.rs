use std::fmt::Display;

use crate::file::File;
use crate::rank::Rank;
use crate::utilities::{QueenLikeMoveDirection, SquaresToMasks};
use crate::{Bitboard, Color};

// Full-board masks for each BR→TL and BL→TR diagonal (15 lines each); used to resolve which line a square lies on.
pub(crate) const DIAGONALS_BR_TO_TL: [Bitboard; 15] = [
    0x0000000000000001,
    0x0000000000000102,
    0x0000000000010204,
    0x0000000001020408,
    0x0000000102040810,
    0x0000010204081020,
    0x0001020408102040,
    0x0102040810204080,
    0x0204081020408000,
    0x0408102040800000,
    0x0810204080000000,
    0x1020408000000000,
    0x2040800000000000,
    0x4080000000000000,
    0x8000000000000000,
];

pub(crate) const DIAGONALS_BL_TO_TR: [Bitboard; 15] = [
    0x0000000000000080,
    0x0000000000008040,
    0x0000000000804020,
    0x0000000080402010,
    0x0000008040201008,
    0x0000804020100804,
    0x0080402010080402,
    0x8040201008040201,
    0x4020100804020100,
    0x2010080402010000,
    0x1008040201000000,
    0x0804020100000000,
    0x0402010000000000,
    0x0201000000000000,
    0x0100000000000000,
];

pub type SquareDelta = i8;

mod private {
    pub const trait Sealed {}
    impl Sealed for super::SquareDelta {}
}

pub const trait SquareDeltaUtils: private::Sealed {
    const UP: SquareDelta;
    const DOWN: SquareDelta;
    const LEFT: SquareDelta;
    const RIGHT: SquareDelta;

    const UP_LEFT: SquareDelta;
    const DOWN_LEFT: SquareDelta;
    const UP_RIGHT: SquareDelta;
    const DOWN_RIGHT: SquareDelta;

    fn from_perspective(self, color: Color) -> SquareDelta;
}

impl const SquareDeltaUtils for SquareDelta {
    const UP: SquareDelta = -8;
    const DOWN: SquareDelta = 8;
    const LEFT: SquareDelta = -1;
    const RIGHT: SquareDelta = 1;

    const UP_LEFT: SquareDelta = SquareDelta::UP + SquareDelta::LEFT;
    const DOWN_LEFT: SquareDelta = SquareDelta::DOWN + SquareDelta::LEFT;
    const UP_RIGHT: SquareDelta = SquareDelta::UP + SquareDelta::RIGHT;
    const DOWN_RIGHT: SquareDelta = SquareDelta::DOWN + SquareDelta::RIGHT;

    fn from_perspective(self, color: Color) -> SquareDelta {
        self * match color {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Represents a square on the chess board.
pub enum Square {
    A8 = 0,
    B8 = 1,
    C8 = 2,
    D8 = 3,
    E8 = 4,
    F8 = 5,
    G8 = 6,
    H8 = 7,
    A7 = 8,
    B7 = 9,
    C7 = 10,
    D7 = 11,
    E7 = 12,
    F7 = 13,
    G7 = 14,
    H7 = 15,
    A6 = 16,
    B6 = 17,
    C6 = 18,
    D6 = 19,
    E6 = 20,
    F6 = 21,
    G6 = 22,
    H6 = 23,
    A5 = 24,
    B5 = 25,
    C5 = 26,
    D5 = 27,
    E5 = 28,
    F5 = 29,
    G5 = 30,
    H5 = 31,
    A4 = 32,
    B4 = 33,
    C4 = 34,
    D4 = 35,
    E4 = 36,
    F4 = 37,
    G4 = 38,
    H4 = 39,
    A3 = 40,
    B3 = 41,
    C3 = 42,
    D3 = 43,
    E3 = 44,
    F3 = 45,
    G3 = 46,
    H3 = 47,
    A2 = 48,
    B2 = 49,
    C2 = 50,
    D2 = 51,
    E2 = 52,
    F2 = 53,
    G2 = 54,
    H2 = 55,
    A1 = 56,
    B1 = 57,
    C1 = 58,
    D1 = 59,
    E1 = 60,
    F1 = 61,
    G1 = 62,
    H1 = 63,
}

impl Square {
    /// Board index `0..64` → square (A8→0 … H1→63), same layout as [`File::from_u8`] / [`Rank::from_u8`].
    ///
    /// **Contract:** `index` must be `< 64`. In debug builds, invalid values panic; in release, invalid
    /// values are **undefined behavior** (invalid enum bit pattern). Use FEN parsing for validated input
    /// from text.
    #[inline]
    pub const fn from_u8(index: u8) -> Square {
        debug_assert!(index < 64);
        unsafe { std::mem::transmute::<u8, Square>(index) }
    }

    /// Returns a square from a bitboard with **exactly one** bit set, or `None` if the mask is empty or has multiple bits.
    #[inline]
    pub const fn from_bitboard(bitboard: Bitboard) -> Option<Square> {
        if bitboard == 0 || !bitboard.is_power_of_two() {
            return None;
        }
        Some(Square::from_u8(bitboard.leading_zeros() as u8))
    }

    /// Returns the square for the given [`Rank`] and [`File`] (same layout as chmog `fromRankAndFile`).
    #[inline]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Square {
        Square::from_u8((7 - rank as u8) * 8 + file as u8)
    }

    /// Returns the bitboard mask for the square.
    pub const fn mask(&self) -> Bitboard {
        1 << (63 - *self as u8)
    }

    pub const fn file(&self) -> File {
        File::from_u8(*self as u8 % 8)
    }

    pub const fn rank(&self) -> Rank {
        Rank::from_u8(7 - *self as u8 / 8)
    }

    /// Returns the combined file and rank mask for the square.
    pub const fn orthogonals_mask(&self) -> Bitboard {
        self.file().mask() | self.rank().mask()
    }

    /// Returns the combined diagonals mask for the square (both `/` and `\` diagonals through this square).
    pub fn diagonals_mask(&self) -> Bitboard {
        DIAGONALS_MASK_LOOKUP.get(*self)
    }

    /// Returns the combined orthogonals and diagonals mask for the square.
    pub fn orthogonals_and_diagonals_mask(&self) -> Bitboard {
        self.orthogonals_mask() | self.diagonals_mask()
    }

    /// True if `other` lies on either diagonal through this square (including `other == self`).
    pub const fn is_diagonal_to(&self, other: Square) -> bool {
        diagonals_union_impl(*self) & other.mask() != 0
    }

    /// Returns whether the square is on the same rank, file, or diagonal as another square (including `other == self`).
    pub const fn is_on_same_line_as(&self, other: Square) -> bool {
        same_line(*self, other)
    }

    /// Offset from this square by `delta` in rank-major index space, or `None` if the sum is outside `0..64`.
    /// Callers stepping orthogonally or diagonally should still enforce file/rank edges; see [`Square::up`], etc.
    pub const fn relative(self, delta: SquareDelta) -> Option<Square> {
        let idx = self as u8 as i16 + delta as i16;
        if idx >= 0 && idx <= 63 {
            Some(Square::from_u8(idx as u8))
        } else {
            None
        }
    }

    /// Returns the square above the current square, if it exists.
    pub const fn up(&self) -> Option<Square> {
        if self.rank() == Rank::Eight {
            None
        } else {
            self.relative(SquareDelta::UP)
        }
    }

    /// Returns the square below the current square, if it exists.
    pub const fn down(&self) -> Option<Square> {
        if self.rank() == Rank::One {
            None
        } else {
            self.relative(SquareDelta::DOWN)
        }
    }

    /// Returns the square to the left of the current square, if it exists.
    pub const fn left(&self) -> Option<Square> {
        if self.file() == File::A {
            None
        } else {
            self.relative(SquareDelta::LEFT)
        }
    }

    /// Returns the square to the right of the current square, if it exists.
    pub const fn right(&self) -> Option<Square> {
        if self.file() == File::H {
            None
        } else {
            self.relative(SquareDelta::RIGHT)
        }
    }

    /// Returns the square northwest of the current square, if it exists.
    pub const fn up_left(&self) -> Option<Square> {
        if self.rank() == Rank::Eight || self.file() == File::A {
            None
        } else {
            self.relative(SquareDelta::UP + SquareDelta::LEFT)
        }
    }

    /// Returns the square northeast of the current square, if it exists.
    pub const fn up_right(&self) -> Option<Square> {
        if self.rank() == Rank::Eight || self.file() == File::H {
            None
        } else {
            self.relative(SquareDelta::UP + SquareDelta::RIGHT)
        }
    }

    /// Returns the square southwest of the current square, if it exists.
    pub const fn down_left(&self) -> Option<Square> {
        if self.rank() == Rank::One || self.file() == File::A {
            None
        } else {
            self.relative(SquareDelta::DOWN + SquareDelta::LEFT)
        }
    }

    /// Returns the square southeast of the current square, if it exists.
    pub const fn down_right(&self) -> Option<Square> {
        if self.rank() == Rank::One || self.file() == File::H {
            None
        } else {
            self.relative(SquareDelta::DOWN + SquareDelta::RIGHT)
        }
    }

    pub const fn neighbor_in_direction(self, direction: QueenLikeMoveDirection) -> Option<Square> {
        match direction {
            QueenLikeMoveDirection::Up => self.up(),
            QueenLikeMoveDirection::Down => self.down(),
            QueenLikeMoveDirection::Right => self.right(),
            QueenLikeMoveDirection::Left => self.left(),
            QueenLikeMoveDirection::UpRight => self.up_right(),
            QueenLikeMoveDirection::UpLeft => self.up_left(),
            QueenLikeMoveDirection::DownRight => self.down_right(),
            QueenLikeMoveDirection::DownLeft => self.down_left(),
        }
    }

    /// Returns the square corresponding to the current square, but as seen from the opposite side of the board.
    pub const fn rotated_perspective(&self) -> Square {
        Square::from_u8(63 - *self as u8)
    }

    /// Returns the character corresponding to the file of the square.
    pub const fn file_char(&self) -> char {
        (b'a' + self.file() as u8) as char
    }

    /// Returns the character corresponding to the rank of the square.
    pub const fn rank_char(&self) -> char {
        (b'1' + self.rank() as u8) as char
    }

    /// Returns a string representing the square in algebraic notation.
    pub const fn algebraic(&self) -> &str {
        Self::ALL_ALGEBRAIC[*self as usize]
    }

    /// An array of all possible squares, ordered from A8 to H1, left to right, top to bottom (numerically ascending).
    pub const ALL: [Square; 64] = [
        Square::A8,
        Square::B8,
        Square::C8,
        Square::D8,
        Square::E8,
        Square::F8,
        Square::G8,
        Square::H8,
        Square::A7,
        Square::B7,
        Square::C7,
        Square::D7,
        Square::E7,
        Square::F7,
        Square::G7,
        Square::H7,
        Square::A6,
        Square::B6,
        Square::C6,
        Square::D6,
        Square::E6,
        Square::F6,
        Square::G6,
        Square::H6,
        Square::A5,
        Square::B5,
        Square::C5,
        Square::D5,
        Square::E5,
        Square::F5,
        Square::G5,
        Square::H5,
        Square::A4,
        Square::B4,
        Square::C4,
        Square::D4,
        Square::E4,
        Square::F4,
        Square::G4,
        Square::H4,
        Square::A3,
        Square::B3,
        Square::C3,
        Square::D3,
        Square::E3,
        Square::F3,
        Square::G3,
        Square::H3,
        Square::A2,
        Square::B2,
        Square::C2,
        Square::D2,
        Square::E2,
        Square::F2,
        Square::G2,
        Square::H2,
        Square::A1,
        Square::B1,
        Square::C1,
        Square::D1,
        Square::E1,
        Square::F1,
        Square::G1,
        Square::H1,
    ];

    pub const ALL_ALGEBRAIC: [&'static str; 64] = [
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", "a7", "b7", "c7", "d7", "e7", "f7", "g7",
        "h7", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", "a5", "b5", "c5", "d5", "e5", "f5",
        "g5", "h5", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", "a3", "b3", "c3", "d3", "e3",
        "f3", "g3", "h3", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", "a1", "b1", "c1", "d1",
        "e1", "f1", "g1", "h1",
    ];
}

const fn ascending_diagonal_mask_impl(square: Square) -> Bitboard {
    let mask = square.mask();
    let mut i = 0;
    while i < 15 {
        if DIAGONALS_BR_TO_TL[i] & mask != 0 {
            return DIAGONALS_BR_TO_TL[i];
        }
        i += 1;
    }
    0
}

const fn descending_diagonal_mask_impl(square: Square) -> Bitboard {
    let mask = square.mask();
    let mut i = 0;
    while i < 15 {
        if DIAGONALS_BL_TO_TR[i] & mask != 0 {
            return DIAGONALS_BL_TO_TR[i];
        }
        i += 1;
    }
    0
}

const fn diagonals_union_impl(square: Square) -> Bitboard {
    ascending_diagonal_mask_impl(square) | descending_diagonal_mask_impl(square)
}

/// Whether `sq2` lies on a rank, file, or diagonal through `sq1` (compile-time friendly).
pub(crate) const fn same_line(sq1: Square, sq2: Square) -> bool {
    (sq1.orthogonals_mask() | diagonals_union_impl(sq1)) & sq2.mask() != 0
}

const DIAGONALS_MASK_DATA: [Bitboard; 64] = {
    let mut arr = [0u64; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = diagonals_union_impl(Square::from_u8(i));
        i += 1;
    }
    arr
};

static DIAGONALS_MASK_LOOKUP: SquaresToMasks = SquaresToMasks::from_array(DIAGONALS_MASK_DATA);

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.algebraic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{File, Rank};

    #[test]
    fn test_square_values() {
        // Test corners
        assert_eq!(Square::A8 as u8, 0);
        assert_eq!(Square::H8 as u8, 7);
        assert_eq!(Square::A1 as u8, 56);
        assert_eq!(Square::H1 as u8, 63);

        // Test middle squares
        assert_eq!(Square::E4 as u8, 36);
        assert_eq!(Square::D5 as u8, 27);
    }

    #[test]
    fn test_from_square_number() {
        assert_eq!(Square::from_u8(0), Square::A8);
        assert_eq!(Square::from_u8(7), Square::H8);
        assert_eq!(Square::from_u8(56), Square::A1);
        assert_eq!(Square::from_u8(63), Square::H1);
        assert_eq!(Square::from_u8(36), Square::E4);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn test_from_u8_out_of_range_debug_panics() {
        let r = std::panic::catch_unwind(|| {
            let _ = Square::from_u8(64);
        });
        assert!(r.is_err(), "from_u8(64) should panic in debug builds");
    }

    #[test]
    fn test_from_rank_and_file() {
        assert_eq!(Square::from_rank_and_file(Rank::Eight, File::A), Square::A8);
        assert_eq!(Square::from_rank_and_file(Rank::Eight, File::H), Square::H8);
        assert_eq!(Square::from_rank_and_file(Rank::One, File::A), Square::A1);
        assert_eq!(Square::from_rank_and_file(Rank::One, File::H), Square::H1);
        assert_eq!(Square::from_rank_and_file(Rank::Four, File::E), Square::E4);
    }

    #[test]
    fn test_from_bitboard_single_bit() {
        assert_eq!(Square::from_bitboard(Square::E4.mask()), Some(Square::E4));
        assert_eq!(Square::from_bitboard(0), None);
        assert_eq!(Square::from_bitboard(3), None);
    }

    #[test]
    fn test_same_line_matches_predicates() {
        for a in Square::ALL {
            for b in Square::ALL {
                assert_eq!(a.is_on_same_line_as(b), same_line(a, b));
                assert_eq!(a.is_diagonal_to(b), diagonals_union_impl(a) & b.mask() != 0);
            }
        }
    }

    #[test]
    fn test_get_mask() {
        assert_eq!(Square::A8.mask(), 1u64 << 63);
        assert_eq!(Square::H1.mask(), 1u64);
        assert_eq!(Square::E4.mask(), 1u64 << 27);
    }

    #[test]
    fn test_get_file() {
        assert_eq!(Square::A8.file() as u8, 0);
        assert_eq!(Square::H8.file() as u8, 7);
        assert_eq!(Square::E4.file() as u8, 4);
    }

    #[test]
    fn test_get_file_mask() {
        let a_file_mask = File::from_u8(Square::A1.file() as u8).mask();
        let h_file_mask = File::from_u8(Square::H1.file() as u8).mask();

        assert_eq!(a_file_mask, File::A.mask());
        assert_eq!(h_file_mask, File::H.mask());

        assert_eq!(
            File::from_u8(Square::A1.file() as u8).mask(),
            File::from_u8(Square::A8.file() as u8).mask()
        );
        assert_eq!(
            File::from_u8(Square::H1.file() as u8).mask(),
            File::from_u8(Square::H8.file() as u8).mask()
        );
    }

    #[test]
    fn test_get_rank() {
        assert_eq!(Square::A8.rank() as u8, 7);
        assert_eq!(Square::A1.rank() as u8, 0);
        assert_eq!(Square::E4.rank() as u8, 3);
    }

    #[test]
    fn test_get_rank_mask() {
        let rank_1_mask = Rank::from_u8(Square::A1.rank() as u8).mask();
        let rank_8_mask = Rank::from_u8(Square::A8.rank() as u8).mask();

        assert_eq!(rank_1_mask, Rank::One.mask());
        assert_eq!(rank_8_mask, Rank::Eight.mask());

        assert_eq!(
            Rank::from_u8(Square::A1.rank() as u8).mask(),
            Rank::from_u8(Square::H1.rank() as u8).mask()
        );
        assert_eq!(
            Rank::from_u8(Square::A8.rank() as u8).mask(),
            Rank::from_u8(Square::H8.rank() as u8).mask()
        );
    }

    #[test]
    fn test_rotated_perspective() {
        assert_eq!(Square::A8.rotated_perspective(), Square::H1);
        assert_eq!(Square::H8.rotated_perspective(), Square::A1);
        assert_eq!(Square::E4.rotated_perspective(), Square::D5);
        assert_eq!(Square::A1.rotated_perspective(), Square::H8);
    }

    #[test]
    fn test_get_file_char() {
        assert_eq!(Square::A1.file_char(), 'a');
        assert_eq!(Square::H8.file_char(), 'h');
        assert_eq!(Square::E4.file_char(), 'e');
    }

    #[test]
    fn test_get_rank_char() {
        assert_eq!(Square::A1.rank_char(), '1');
        assert_eq!(Square::H8.rank_char(), '8');
        assert_eq!(Square::E4.rank_char(), '4');
    }

    #[test]
    fn test_readable() {
        assert_eq!(Square::A1.algebraic(), "a1");
        assert_eq!(Square::H8.algebraic(), "h8");
        assert_eq!(Square::E4.algebraic(), "e4");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Square::A1), "a1");
        assert_eq!(format!("{}", Square::H8), "h8");
        assert_eq!(format!("{}", Square::E4), "e4");
    }

    #[test]
    fn test_iter_all() {
        let all_squares = Square::ALL.into_iter().collect::<Vec<Square>>();
        assert_eq!(all_squares.len(), 64);
        assert_eq!(all_squares[0], Square::A8);
        assert_eq!(all_squares[63], Square::H1);
    }
}
