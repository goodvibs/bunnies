use crate::Square;
use crate::utilities::{BitCombinationsIterator, MaskBitsIterator, MaskSquaresIterator};

/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;

pub trait BitboardUtils {
    /// Returns the mask of squares between two squares, inclusive/exclusive??.
    fn between_squares(&self, sq1: Square, sq2: Square) -> Bitboard;

    /// Returns an iterator that generates the set bits of the bitboard.
    fn iter_set_bits_as_masks(self) -> MaskBitsIterator;

    /// Returns an iterator that generates the squares of the bitboard.
    fn iter_set_bits_as_squares(self) -> MaskSquaresIterator;

    /// Returns an iterator that generates all possible set bit combinations of the bitboard.
    fn iter_bit_combinations(self) -> BitCombinationsIterator;
}

impl BitboardUtils for Bitboard {
    fn between_squares(&self, sq1: Square, sq2: Square) -> Bitboard {
        todo!()
    }
    
    fn iter_set_bits_as_masks(self) -> MaskBitsIterator {
        self.into()
    }

    fn iter_set_bits_as_squares(self) -> MaskSquaresIterator {
        self.into()
    }

    fn iter_bit_combinations(self) -> BitCombinationsIterator {
        self.into()
    }
}