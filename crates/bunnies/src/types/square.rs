//! Chess board squares (A1-H8) and square geometry operations.

use std::fmt::Display;

use super::{
    bitboard::Bitboard,
    file::File,
    rank::Rank,
    square_delta::{SquareDelta, SquareDeltaUtils},
};
use crate::{
    types::{Array, BitboardUtils, QueenLikeMoveDirection},
    utilities::{IterableEnum, impl_u8_conversions},
};

const fn resolve_square_mask(maybe_square: Option<Square>) -> Bitboard {
    match maybe_square {
        Some(square) => square.mask(),
        None => 0,
    }
}

const fn build_diagonals(
    start: Square,
    next_a: QueenLikeMoveDirection,
    next_b: QueenLikeMoveDirection,
) -> Array<Bitboard, 15> {
    let mut builder = Array([0; 15]);

    let mut i = 0;
    let mut mask = start.mask();
    while i < 15 {
        builder[i] = mask;
        let mut new_mask = 0;
        for square in mask.iter_set_bits_as_squares() {
            new_mask |= resolve_square_mask(square.neighbor_in_direction(next_a))
                | resolve_square_mask(square.neighbor_in_direction(next_b));
        }
        mask = new_mask;
        i += 1;
    }

    builder
}

const DIAGONALS_BR_TO_TL: Array<Bitboard, 15> = build_diagonals(
    Square::H1,
    QueenLikeMoveDirection::Left,
    QueenLikeMoveDirection::Up,
);

const DIAGONALS_BL_TO_TR: Array<Bitboard, 15> = build_diagonals(
    Square::A1,
    QueenLikeMoveDirection::Right,
    QueenLikeMoveDirection::Up,
);

/// A chess square using 0..63 indexing (0=A8, 63=H1) matching the bunnies bitboard layout.
///
/// The ordering is rank-major from Black's perspective (A8..H8, then A7..H7, etc.),
/// which naturally maps to bitboard representation where bit 63 = A8 and bit 0 = H1.
#[repr(u8)]
#[derive(Clone, Copy, Eq, Debug, std::marker::ConstParamTy)]
#[derive_const(PartialEq)]
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
    /// Extracts the single square from a bitboard with exactly one bit set.
    ///
    /// Returns `None` if the mask is empty or has multiple bits set.
    #[inline]
    pub const fn from_bitboard(bitboard: Bitboard) -> Option<Square> {
        if bitboard == 0 || !bitboard.is_power_of_two() {
            return None;
        }
        Some({
            let value = bitboard.leading_zeros() as u8;
            unsafe { Self::try_from(value).unwrap_unchecked() }
        })
    }

    /// Constructs a square from rank and file (same layout as chmog `fromRankAndFile`).
    #[inline]
    pub const fn from_rank_and_file(rank: Rank, file: File) -> Square {
        {
            let value = (7 - rank as u8) * 8 + file as u8;
            unsafe { Self::try_from(value).unwrap_unchecked() }
        }
    }

    /// Returns the bitboard mask with only this square's bit set.
    pub const fn mask(self) -> Bitboard {
        1 << (63 - self as u8)
    }

    /// Returns the file (column) of this square.
    pub const fn file(self) -> File {
        {
            let value = self as u8 % 8;
            unsafe { File::try_from(value).unwrap_unchecked() }
        }
    }

    /// Returns the rank (row) of this square.
    pub const fn rank(self) -> Rank {
        unsafe { (7 - self as u8 / 8).try_into().unwrap_unchecked() }
    }

    /// Bitboard mask of the rank and file passing through this square.
    pub const fn orthogonals_mask(self) -> Bitboard {
        self.file().mask() | self.rank().mask()
    }

    /// Bitboard mask of both diagonals passing through this square.
    pub const fn diagonals_mask(self) -> Bitboard {
        DIAGONALS_MASK_LOOKUP[self as usize]
    }

    /// Combined mask of orthogonals and diagonals (queen-like lines from this square).
    pub fn orthogonals_and_diagonals_mask(self) -> Bitboard {
        self.orthogonals_mask() | self.diagonals_mask()
    }

    /// True if `other` lies on either diagonal through this square.
    pub const fn is_diagonal_to(self, other: Square) -> bool {
        diagonals_union_impl(self) & other.mask() != 0
    }

    /// True if `other` is on same rank, file, or diagonal as this square.
    pub const fn is_on_same_line_as(self, other: Square) -> bool {
        same_line(self, other)
    }

    /// Square offset by `delta`, or `None` if outside the board.
    ///
    /// For orthogonal/diagonal steps, prefer the named methods ([`up`](Self::up), [`down`](Self::down), etc.)
    /// which correctly handle board edges.
    pub const fn relative(self, delta: SquareDelta) -> Option<Square> {
        let idx = self as u8 as i16 + delta as i16;
        if idx >= 0 && idx <= 63 {
            Some({
                let value = idx as u8;
                unsafe { Self::try_from(value).unwrap_unchecked() }
            })
        } else {
            None
        }
    }

    /// Square above this one (toward rank 8), or `None` at the top edge.
    pub const fn up(self) -> Option<Square> {
        if self.rank() == Rank::Eight {
            None
        } else {
            self.relative(SquareDelta::UP)
        }
    }

    /// Square below this one (toward rank 1), or `None` at the bottom edge.
    pub const fn down(self) -> Option<Square> {
        if self.rank() == Rank::One {
            None
        } else {
            self.relative(SquareDelta::DOWN)
        }
    }

    /// Square to the left (toward file A), or `None` at the left edge.
    pub const fn left(self) -> Option<Square> {
        if self.file() == File::A {
            None
        } else {
            self.relative(SquareDelta::LEFT)
        }
    }

    /// Square to the right (toward file H), or `None` at the right edge.
    pub const fn right(self) -> Option<Square> {
        if self.file() == File::H {
            None
        } else {
            self.relative(SquareDelta::RIGHT)
        }
    }

    /// Square diagonally up-left, or `None` at either edge.
    pub const fn up_left(self) -> Option<Square> {
        if self.rank() == Rank::Eight || self.file() == File::A {
            None
        } else {
            self.relative(SquareDelta::UP + SquareDelta::LEFT)
        }
    }

    /// Square diagonally up-right, or `None` at either edge.
    pub const fn up_right(self) -> Option<Square> {
        if self.rank() == Rank::Eight || self.file() == File::H {
            None
        } else {
            self.relative(SquareDelta::UP + SquareDelta::RIGHT)
        }
    }

    /// Square diagonally down-left, or `None` at either edge.
    pub const fn down_left(self) -> Option<Square> {
        if self.rank() == Rank::One || self.file() == File::A {
            None
        } else {
            self.relative(SquareDelta::DOWN + SquareDelta::LEFT)
        }
    }

    /// Square diagonally down-right, or `None` at either edge.
    pub const fn down_right(self) -> Option<Square> {
        if self.rank() == Rank::One || self.file() == File::H {
            None
        } else {
            self.relative(SquareDelta::DOWN + SquareDelta::RIGHT)
        }
    }

    /// Adjacent square in the given direction, or `None` at board edge.
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

    /// The square rotated 180 degrees (view from opponent's perspective).
    pub const fn rotated_perspective(self) -> Square {
        {
            let value = 63 - self as u8;
            unsafe { Self::try_from(value).unwrap_unchecked() }
        }
    }

    /// Lowercase file letter ('a'-'h').
    pub const fn file_char(self) -> char {
        (b'a' + self.file() as u8) as char
    }

    /// Rank digit character ('1'-'8').
    pub const fn rank_char(self) -> char {
        (b'1' + self.rank() as u8) as char
    }

    /// Algebraic notation string (e.g., "e4", "h1").
    pub const fn algebraic(self) -> &'static str {
        Self::ALL_ALGEBRAIC[self as usize]
    }

    /// Lookup table of all algebraic notations (A8, B8, ..., H1).
    pub const ALL_ALGEBRAIC: Array<&'static str, 64> = Array([
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8", "a7", "b7", "c7", "d7", "e7", "f7", "g7",
        "h7", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6", "a5", "b5", "c5", "d5", "e5", "f5",
        "g5", "h5", "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4", "a3", "b3", "c3", "d3", "e3",
        "f3", "g3", "h3", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", "a1", "b1", "c1", "d1",
        "e1", "f1", "g1", "h1",
    ]);
}

const fn ascending_diagonal_mask_impl(square: Square) -> Bitboard {
    let mask = square.mask();
    for diagonal in DIAGONALS_BR_TO_TL {
        if diagonal & mask != 0 {
            return diagonal;
        }
    }
    0
}

const fn descending_diagonal_mask_impl(square: Square) -> Bitboard {
    let mask = square.mask();
    for diagonal in DIAGONALS_BL_TO_TR {
        if diagonal & mask != 0 {
            return diagonal;
        }
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

static DIAGONALS_MASK_LOOKUP: Array<Bitboard, 64> = Array({
    let mut arr = [0u64; 64];
    for square in <Square as IterableEnum<64>>::ALL {
        arr[square as usize] = diagonals_union_impl(square);
    }
    arr
});

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.algebraic())
    }
}

impl const IterableEnum<64> for Square {
    const ALL: Array<Square, 64> = Array([
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
    ]);
}

impl_u8_conversions!(Square, 64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{File, Rank};

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
        assert_eq!(Square::try_from(0).unwrap(), Square::A8);
        assert_eq!(Square::try_from(7).unwrap(), Square::H8);
        assert_eq!(Square::try_from(56).unwrap(), Square::A1);
        assert_eq!(Square::try_from(63).unwrap(), Square::H1);
        assert_eq!(Square::try_from(36).unwrap(), Square::E4);
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
        let a_file_mask = {
            let value = Square::A1.file() as u8;
            unsafe { File::try_from(value).unwrap_unchecked() }
        }
        .mask();
        let h_file_mask = {
            let value = Square::H1.file() as u8;
            unsafe { File::try_from(value).unwrap_unchecked() }
        }
        .mask();

        assert_eq!(a_file_mask, File::A.mask());
        assert_eq!(h_file_mask, File::H.mask());

        assert_eq!(
            unsafe { File::try_from(Square::A1.file() as u8).unwrap_unchecked() }.mask(),
            unsafe { File::try_from(Square::A8.file() as u8).unwrap_unchecked() }.mask(),
        );
        assert_eq!(
            unsafe { File::try_from(Square::H1.file() as u8).unwrap_unchecked() }.mask(),
            unsafe { File::try_from(Square::H8.file() as u8).unwrap_unchecked() }.mask(),
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
        let rank_1_mask = {
            let value = Square::A1.rank() as u8;
            unsafe { Rank::try_from(value).unwrap_unchecked() }
        }
        .mask();
        let rank_8_mask = {
            let value = Square::A8.rank() as u8;
            unsafe { Rank::try_from(value).unwrap_unchecked() }
        }
        .mask();

        assert_eq!(rank_1_mask, Rank::One.mask());
        assert_eq!(rank_8_mask, Rank::Eight.mask());

        assert_eq!(
            Rank::try_from(Square::A1.rank() as u8).unwrap().mask(),
            Rank::try_from(Square::H1.rank() as u8).unwrap().mask()
        );
        assert_eq!(
            Rank::try_from(Square::A8.rank() as u8).unwrap().mask(),
            Rank::try_from(Square::H8.rank() as u8).unwrap().mask()
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
}
