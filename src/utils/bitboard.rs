use crate::utils::Square;

/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;

#[derive(Debug, Clone)]
/// An iterator that generates the set bits of a bitboard.
pub struct SetBitMaskIterator {
    current_mask: Bitboard,
}

impl From<Bitboard> for SetBitMaskIterator {
    fn from(mask: Bitboard) -> Self {
        SetBitMaskIterator {
            current_mask: mask,
        }
    }
}

impl Iterator for SetBitMaskIterator {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_mask == 0 {
            return None;
        }

        let ls1b_mask = self.current_mask & self.current_mask.wrapping_neg();  // Isolate the least significant set bit
        self.current_mask &= !ls1b_mask;  // Clear the least significant set bit

        Some(ls1b_mask)
    }
}

/// Returns an iterator that generates the set bits of a bitboard.
pub fn iter_set_bits(mask: Bitboard) -> SetBitMaskIterator {
    mask.into()
}

#[derive(Debug, Clone)]
pub struct SquaresFromMaskIterator {
    current_mask: Bitboard,
}

impl From<Bitboard> for SquaresFromMaskIterator {
    fn from(mask: Bitboard) -> Self {
        SquaresFromMaskIterator {
            current_mask: mask,
        }
    }
}

impl Iterator for SquaresFromMaskIterator {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_mask == 0 {
            return None;
        }

        let ls1b_mask = self.current_mask & self.current_mask.wrapping_neg();  // Isolate the least significant set bit
        self.current_mask &= !ls1b_mask;  // Clear the least significant set bit

        unsafe {
            Some(Square::from_bitboard(ls1b_mask))
        }
    }
}

/// Returns an iterator that generates the squares of a bitboard.
pub fn iter_squares_from_mask(mask: Bitboard) -> SquaresFromMaskIterator {
    mask.into()
}

#[derive(Debug, Clone)]
/// An iterator that generates all possible set bit combinations of a bitboard.
pub struct BitCombinationsIterator {
    set: Bitboard,
    subset: Bitboard,
    finished: bool,
}

impl From<Bitboard> for BitCombinationsIterator {
    fn from(set: Bitboard) -> Self {
        BitCombinationsIterator {
            set,
            subset: 0,
            finished: set == 0,
        }
    }
}

impl Iterator for BitCombinationsIterator {
    type Item = Bitboard;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        let current = self.subset;
        self.subset = self.subset.wrapping_sub(self.set) & self.set;

        // Once we generate the 0 subset again, we're done
        if self.subset == 0 && current != 0 {
            self.finished = true;
        }

        Some(current)
    }
}

/// Returns an iterator that generates all possible set bit combinations of a bitboard.
pub fn iter_bit_combinations(mask: Bitboard) -> BitCombinationsIterator {
    mask.into()
}

/// Prints a Bitboard as a binary number.
pub fn print_bb(bb: Bitboard) {
    for i in 0..8 {
        let shift_amt = 8 * (7 - i);
        println!("{:08b}", (bb & (0xFF << shift_amt)) >> shift_amt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bit_combinations() {
        // Test with an empty bitmask
        let mask = 0;
        let expected: Vec<Bitboard> = vec![];
        let result: Vec<Bitboard> = iter_bit_combinations(mask).collect();
        assert_eq!(result, expected);

        // Test with a bitmask that has one bit set
        let mask = 0b0001;
        let expected: Vec<Bitboard> = vec![0b0000, 0b0001];
        let result: Vec<Bitboard> = iter_bit_combinations(mask).collect();
        assert_eq!(result, expected);

        // Test with a bitmask that has multiple bits set
        let mask = 0b1010;
        let expected: Vec<Bitboard> = vec![0b0000, 0b0010, 0b1000, 0b1010];
        let result: Vec<Bitboard> = iter_bit_combinations(mask).collect();
        assert_eq!(result, expected);

        // Test with a full bitmask (all bits set for a small size)
        let mask = 0b1111;
        let expected: Vec<Bitboard> = vec![
            0b0000, 0b0001, 0b0010, 0b0011,
            0b0100, 0b0101, 0b0110, 0b0111,
            0b1000, 0b1001, 0b1010, 0b1011,
            0b1100, 0b1101, 0b1110, 0b1111,
        ];
        let result: Vec<Bitboard> = iter_bit_combinations(mask).collect();
        assert_eq!(result, expected);
    }
}