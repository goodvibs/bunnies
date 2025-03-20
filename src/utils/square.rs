use std::fmt::Display;
use std::ops::Range;
use crate::utils::Bitboard;
use crate::utils::charboard::SQUARE_NAMES;
use crate::utils::Color;
use crate::utils::masks::{FILES, RANKS};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    pub const unsafe fn from(square_number: u8) -> Square {
        assert!(square_number < 64, "Square number out of bounds");
        std::mem::transmute::<u8, Square>(square_number)
    }
    
    pub const unsafe fn from_rank_file(rank: u8, file: u8) -> Square {
        assert!(rank < 8 && file < 8, "Rank or file out of bounds");
        std::mem::transmute::<u8, Square>((7 - rank) * 8 + file)
    }

    pub const fn get_mask(&self) -> Bitboard {
        1 << (63 - *self as u8)
    }

    pub const fn get_file(&self) -> u8 {
        *self as u8 % 8
    }

    pub const fn get_file_mask(&self) -> Bitboard {
        FILES[self.get_file() as usize]
    }

    pub const fn get_rank(&self) -> u8 {
        7 - *self as u8 / 8
    }

    pub const fn get_rank_mask(&self) -> Bitboard {
        RANKS[self.get_rank() as usize]
    }
    
    pub const fn up(&self) -> Option<Square> {
        if self.get_rank() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 8) })
        }
    }
    
    pub const fn down(&self) -> Option<Square> {
        if self.get_rank() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 8) })
        }
    }
    
    pub const fn left(&self) -> Option<Square> {
        if self.get_file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 1) })
        }
    }
    
    pub const fn right(&self) -> Option<Square> {
        if self.get_file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 1) })
        }
    }
    
    pub const fn up_left(&self) -> Option<Square> {
        if self.get_rank() == 7 || self.get_file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 9) })
        }
    }
    
    pub const fn up_right(&self) -> Option<Square> {
        if self.get_rank() == 7 || self.get_file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 - 7) })
        }
    }
    
    pub const fn down_left(&self) -> Option<Square> {
        if self.get_rank() == 0 || self.get_file() == 0 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 7) })
        }
    }
    
    pub const fn down_right(&self) -> Option<Square> {
        if self.get_rank() == 0 || self.get_file() == 7 {
            None
        } else {
            Some(unsafe { Square::from(*self as u8 + 9) })
        }
    }
    
    pub const fn reflect_rank(&self) -> Square {
        unsafe { Square::from((self.get_rank() * 8) + self.get_file()) }
    }
    
    pub const fn rotated_perspective(&self) -> Square {
        unsafe { Square::from(63 - *self as u8) }
    }
    
    pub const fn to_perspective_from_white(&self, desired_perspective: Color) -> Square {
        match desired_perspective {
            Color::White => *self,
            Color::Black => self.rotated_perspective()
        }
    }
    
    pub const fn to_perspective_from_black(&self, desired_perspective: Color) -> Square {
        match desired_perspective {
            Color::White => self.rotated_perspective(),
            Color::Black => *self
        }
    }

    pub const fn get_file_char(&self) -> char {
        (b'a' + self.get_file()) as char
    }

    pub const fn get_rank_char(&self) -> char {
        (b'1' + self.get_rank()) as char
    }

    pub const fn readable(&self) -> &str {
        SQUARE_NAMES[*self as usize]
    }

    pub fn iter_all() -> SquareIter {
        SquareIter { range: 0..64 }
    }
}

impl Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.readable())
    }
}

pub struct SquareIter {
    range: Range<u8>,
}

impl Iterator for SquareIter {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|idx| unsafe { std::mem::transmute(idx) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }
}

impl ExactSizeIterator for SquareIter {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Color;

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
        assert_eq!(Square::A8.get_mask(), 1u64 << 63);
        assert_eq!(Square::H1.get_mask(), 1u64);
        assert_eq!(Square::E4.get_mask(), 1u64 << 27);
    }

    #[test]
    fn test_get_file() {
        assert_eq!(Square::A8.get_file(), 0);
        assert_eq!(Square::H8.get_file(), 7);
        assert_eq!(Square::E4.get_file(), 4);
    }

    #[test]
    fn test_get_file_mask() {
        // This test assumes FILES is correctly implemented
        // Testing that the correct file mask is returned
        let a_file_mask = Square::A1.get_file_mask();
        let h_file_mask = Square::H1.get_file_mask();

        assert_eq!(a_file_mask, FILES[0]);
        assert_eq!(h_file_mask, FILES[7]);

        // Check that all squares in the same file return the same mask
        assert_eq!(Square::A1.get_file_mask(), Square::A8.get_file_mask());
        assert_eq!(Square::H1.get_file_mask(), Square::H8.get_file_mask());
    }

    #[test]
    fn test_get_rank() {
        assert_eq!(Square::A8.get_rank(), 7);
        assert_eq!(Square::A1.get_rank(), 0);
        assert_eq!(Square::E4.get_rank(), 3);
    }

    #[test]
    fn test_get_rank_mask() {
        // This test assumes RANKS is correctly implemented
        // Testing that the correct rank mask is returned
        let rank_1_mask = Square::A1.get_rank_mask();
        let rank_8_mask = Square::A8.get_rank_mask();

        assert_eq!(rank_1_mask, RANKS[0]);
        assert_eq!(rank_8_mask, RANKS[7]);

        // Check that all squares in the same rank return the same mask
        assert_eq!(Square::A1.get_rank_mask(), Square::H1.get_rank_mask());
        assert_eq!(Square::A8.get_rank_mask(), Square::H8.get_rank_mask());
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
    fn test_reflect_rank() {
        assert_eq!(Square::A1.reflect_rank(), Square::A8);
        assert_eq!(Square::H1.reflect_rank(), Square::H8);
        assert_eq!(Square::E4.reflect_rank(), Square::E5);
        assert_eq!(Square::D5.reflect_rank(), Square::D4);
    }

    #[test]
    fn test_rotated_perspective() {
        assert_eq!(Square::A8.rotated_perspective(), Square::H1);
        assert_eq!(Square::H8.rotated_perspective(), Square::A1);
        assert_eq!(Square::E4.rotated_perspective(), Square::D5);
        assert_eq!(Square::A1.rotated_perspective(), Square::H8);
    }

    #[test]
    fn test_perspective_transforms() {
        // White perspective
        assert_eq!(Square::E2.to_perspective_from_white(Color::White), Square::E2);
        assert_eq!(Square::E2.to_perspective_from_white(Color::Black), Square::D7);

        // Black perspective
        assert_eq!(Square::E7.to_perspective_from_black(Color::White), Square::D2);
        assert_eq!(Square::E7.to_perspective_from_black(Color::Black), Square::E7);
    }

    #[test]
    fn test_get_file_char() {
        assert_eq!(Square::A1.get_file_char(), 'a');
        assert_eq!(Square::H8.get_file_char(), 'h');
        assert_eq!(Square::E4.get_file_char(), 'e');
    }

    #[test]
    fn test_get_rank_char() {
        assert_eq!(Square::A1.get_rank_char(), '1');
        assert_eq!(Square::H8.get_rank_char(), '8');
        assert_eq!(Square::E4.get_rank_char(), '4');
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
        let all_squares = Square::iter_all().collect::<Vec<Square>>();
        assert_eq!(all_squares.len(), 64);
        assert_eq!(all_squares[0], Square::A8);
        assert_eq!(all_squares[63], Square::H1);
    }
}