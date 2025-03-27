use std::fmt::Display;
use crate::utils::Bitboard;
use crate::utils::masks::{FILES, RANKS};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// Represents a square on the chess board.
pub enum Square {
    A8=0, B8=1, C8=2, D8=3, E8=4, F8=5, G8=6, H8=7,
    A7=8, B7=9, C7=10, D7=11, E7=12, F7=13, G7=14, H7=15,
    A6=16, B6=17, C6=18, D6=19, E6=20, F6=21, G6=22, H6=23,
    A5=24, B5=25, C5=26, D5=27, E5=28, F5=29, G5=30, H5=31,
    A4=32, B4=33, C4=34, D4=35, E4=36, F4=37, G4=38, H4=39,
    A3=40, B3=41, C3=42, D3=43, E3=44, F3=45, G3=46, H3=47,
    A2=48, B2=49, C2=50, D2=51, E2=52, F2=53, G2=54, H2=55,
    A1=56, B1=57, C1=58, D1=59, E1=60, F1=61, G1=62, H1=63
}

impl Square {
    /// Returns a square from a square number (0-63).
    /// Square numbers are ordered from A8 to H1, left to right, top to bottom.
    /// # Safety
    /// This function is unsafe because it does not check if the square number is out of bounds.
    pub const unsafe fn from(square_number: u8) -> Square {
        assert!(square_number < 64, "Square number out of bounds");
        unsafe { std::mem::transmute::<u8, Square>(square_number) }
    }

    /// Returns a square from a bitboard.
    /// # Safety
    /// This function is unsafe because it does not check if the bitboard has exactly one bit set.
    pub const unsafe fn from_bitboard(bitboard: Bitboard) -> Square {
        unsafe { Square::from(bitboard.leading_zeros() as u8) }
    }
    
    /// Returns a square from a rank and file (0-7).
    /// # Safety
    /// This function is unsafe because it does not check if the rank or file is out of bounds.
    pub const unsafe fn from_rank_file(rank: u8, file: u8) -> Square {
        assert!(rank < 8 && file < 8, "Rank or file out of bounds");
        unsafe { std::mem::transmute::<u8, Square>((7 - rank) * 8 + file) }
    }

    /// Returns the bitboard mask for the square.
    pub const fn mask(&self) -> Bitboard {
        1 << (63 - *self as u8)
    }

    /// Returns the file of the square (0-7).
    pub const fn file(&self) -> u8 {
        *self as u8 % 8
    }

    /// Returns the file mask for the square.
    pub const fn file_mask(&self) -> Bitboard {
        FILES[self.file() as usize]
    }

    /// Returns the rank of the square (0-7).
    pub const fn rank(&self) -> u8 {
        7 - *self as u8 / 8
    }

    /// Returns the rank mask for the square.
    pub const fn rank_mask(&self) -> Bitboard {
        RANKS[self.rank() as usize]
    }
    
    /// Returns the square above the current square, if it exists.
    pub const fn up(&self) -> Option<Square> {
        if self.rank() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 8) })
        }
    }
    
    /// Returns the square below the current square, if it exists.
    pub const fn down(&self) -> Option<Square> {
        if self.rank() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 8) })
        }
    }
    
    /// Returns the square to the left of the current square, if it exists.
    pub const fn left(&self) -> Option<Square> {
        if self.file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 1) })
        }
    }
    
    /// Returns the square to the right of the current square, if it exists.
    pub const fn right(&self) -> Option<Square> {
        if self.file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 1) })
        }
    }
    
    /// Returns the square northwest of the current square, if it exists.
    pub const fn up_left(&self) -> Option<Square> {
        if self.rank() == 7 || self.file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 9) })
        }
    }
    
    /// Returns the square northeast of the current square, if it exists.
    pub const fn up_right(&self) -> Option<Square> {
        if self.rank() == 7 || self.file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 7) })
        }
    }
    
    /// Returns the square southwest of the current square, if it exists.
    pub const fn down_left(&self) -> Option<Square> {
        if self.rank() == 0 || self.file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 7) })
        }
    }
    
    /// Returns the square southeast of the current square, if it exists.
    pub const fn down_right(&self) -> Option<Square> {
        if self.rank() == 0 || self.file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 9) })
        }
    }
    
    /// Returns the square corresponding to the current square, but as seen from the opposite side of the board.
    pub const fn rotated_perspective(&self) -> Square {
        unsafe { Square::from(63 - *self as u8) }
    }

    /// Returns the character corresponding to the file of the square.
    pub const fn file_char(&self) -> char {
        (b'a' + self.file()) as char
    }

    /// Returns the character corresponding to the rank of the square.
    pub const fn rank_char(&self) -> char {
        (b'1' + self.rank()) as char
    }

    /// Returns a string representing the square in algebraic notation.
    pub const fn readable(&self) -> &str {
        Self::ALL_STR[*self as usize]
    }

    /// An array of all possible squares, ordered from A8 to H1, left to right, top to bottom (numerically ascending).
    pub const ALL: [Square; 64] = [
        Square::A8, Square::B8, Square::C8, Square::D8, Square::E8, Square::F8, Square::G8, Square::H8,
        Square::A7, Square::B7, Square::C7, Square::D7, Square::E7, Square::F7, Square::G7, Square::H7,
        Square::A6, Square::B6, Square::C6, Square::D6, Square::E6, Square::F6, Square::G6, Square::H6,
        Square::A5, Square::B5, Square::C5, Square::D5, Square::E5, Square::F5, Square::G5, Square::H5,
        Square::A4, Square::B4, Square::C4, Square::D4, Square::E4, Square::F4, Square::G4, Square::H4,
        Square::A3, Square::B3, Square::C3, Square::D3, Square::E3, Square::F3, Square::G3, Square::H3,
        Square::A2, Square::B2, Square::C2, Square::D2, Square::E2, Square::F2, Square::G2, Square::H2,
        Square::A1, Square::B1, Square::C1, Square::D1, Square::E1, Square::F1, Square::G1, Square::H1
    ];

    pub const ALL_STR: [&'static str; 64] = [
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
        "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
        "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
        "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
        "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
        "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
        "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1"
    ];
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.readable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        unsafe {
            assert_eq!(Square::from(0), Square::A8);
            assert_eq!(Square::from(7), Square::H8);
            assert_eq!(Square::from(56), Square::A1);
            assert_eq!(Square::from(63), Square::H1);
            assert_eq!(Square::from(36), Square::E4);
        }
    }

    #[test]
    #[should_panic(expected = "Square number out of bounds")]
    fn test_from_square_number_out_of_bounds() {
        unsafe {
            let _ = Square::from(64);
        }
    }

    #[test]
    fn test_from_rank_file() {
        unsafe {
            assert_eq!(Square::from_rank_file(7, 0), Square::A8);
            assert_eq!(Square::from_rank_file(7, 7), Square::H8);
            assert_eq!(Square::from_rank_file(0, 0), Square::A1);
            assert_eq!(Square::from_rank_file(0, 7), Square::H1);
            assert_eq!(Square::from_rank_file(3, 4), Square::E4);
        }
    }

    #[test]
    #[should_panic(expected = "Rank or file out of bounds")]
    fn test_from_rank_file_out_of_bounds() {
        unsafe {
            let _ = Square::from_rank_file(8, 0);
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
        assert_eq!(Square::A8.file(), 0);
        assert_eq!(Square::H8.file(), 7);
        assert_eq!(Square::E4.file(), 4);
    }

    #[test]
    fn test_get_file_mask() {
        // This test assumes FILES is correctly implemented
        // Testing that the correct file mask is returned
        let a_file_mask = Square::A1.file_mask();
        let h_file_mask = Square::H1.file_mask();

        assert_eq!(a_file_mask, FILES[0]);
        assert_eq!(h_file_mask, FILES[7]);

        // Check that all squares in the same file return the same mask
        assert_eq!(Square::A1.file_mask(), Square::A8.file_mask());
        assert_eq!(Square::H1.file_mask(), Square::H8.file_mask());
    }

    #[test]
    fn test_get_rank() {
        assert_eq!(Square::A8.rank(), 7);
        assert_eq!(Square::A1.rank(), 0);
        assert_eq!(Square::E4.rank(), 3);
    }

    #[test]
    fn test_get_rank_mask() {
        // This test assumes RANKS is correctly implemented
        // Testing that the correct rank mask is returned
        let rank_1_mask = Square::A1.rank_mask();
        let rank_8_mask = Square::A8.rank_mask();

        assert_eq!(rank_1_mask, RANKS[0]);
        assert_eq!(rank_8_mask, RANKS[7]);

        // Check that all squares in the same rank return the same mask
        assert_eq!(Square::A1.rank_mask(), Square::H1.rank_mask());
        assert_eq!(Square::A8.rank_mask(), Square::H8.rank_mask());
    }

    #[test]
    fn test_up() {
        // Middle of board
        assert_eq!(Square::E4.up(), Some(Square::E5));

        // Edge cases
        assert_eq!(Square::E8.up(), None);
        assert_eq!(Square::A1.up(), Some(Square::A2));
    }

    #[test]
    fn test_down() {
        // Middle of board
        assert_eq!(Square::E4.down(), Some(Square::E3));

        // Edge cases
        assert_eq!(Square::E1.down(), None);
        assert_eq!(Square::A8.down(), Some(Square::A7));
    }

    #[test]
    fn test_left() {
        // Middle of board
        assert_eq!(Square::E4.left(), Some(Square::D4));

        // Edge cases
        assert_eq!(Square::A4.left(), None);
        assert_eq!(Square::H1.left(), Some(Square::G1));
    }

    #[test]
    fn test_right() {
        // Middle of board
        assert_eq!(Square::E4.right(), Some(Square::F4));

        // Edge cases
        assert_eq!(Square::H4.right(), None);
        assert_eq!(Square::A1.right(), Some(Square::B1));
    }

    #[test]
    fn test_diagonal_moves() {
        // Test up_left
        assert_eq!(Square::E4.up_left(), Some(Square::D5));
        assert_eq!(Square::A4.up_left(), None); // Left edge
        assert_eq!(Square::E8.up_left(), None); // Top edge
        assert_eq!(Square::A8.up_left(), None); // Top-left corner

        // Test up_right
        assert_eq!(Square::E4.up_right(), Some(Square::F5));
        assert_eq!(Square::H4.up_right(), None); // Right edge
        assert_eq!(Square::E8.up_right(), None); // Top edge
        assert_eq!(Square::H8.up_right(), None); // Top-right corner

        // Test down_left
        assert_eq!(Square::E4.down_left(), Some(Square::D3));
        assert_eq!(Square::A4.down_left(), None); // Left edge
        assert_eq!(Square::E1.down_left(), None); // Bottom edge
        assert_eq!(Square::A1.down_left(), None); // Bottom-left corner

        // Test down_right
        assert_eq!(Square::E4.down_right(), Some(Square::F3));
        assert_eq!(Square::H4.down_right(), None); // Right edge
        assert_eq!(Square::E1.down_right(), None); // Bottom edge
        assert_eq!(Square::H1.down_right(), None); // Bottom-right corner
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
        assert_eq!(Square::A1.readable(), "a1");
        assert_eq!(Square::H8.readable(), "h8");
        assert_eq!(Square::E4.readable(), "e4");
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