use static_init::dynamic;
use crate::Square;
use crate::utilities::{BitCombinationsIterator, MaskBitsIterator, MaskSquaresIterator, QueenLikeMoveDirection, SquarePairsToMasks};

/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;

pub trait BitboardUtils {
    /// Returns the mask of squares between two squares, inclusive/exclusive??.
    fn between_squares(sq1: Square, sq2: Square) -> Bitboard;

    /// Returns an iterator that generates the set bits of the bitboard.
    fn iter_set_bits_as_masks(self) -> MaskBitsIterator;

    /// Returns an iterator that generates the squares of the bitboard.
    fn iter_set_bits_as_squares(self) -> MaskSquaresIterator;

    /// Returns an iterator that generates all possible set bit combinations of the bitboard.
    fn iter_bit_combinations(self) -> BitCombinationsIterator;
}

impl BitboardUtils for Bitboard {
    fn between_squares(sq1: Square, sq2: Square) -> Bitboard {
        BB_BETWEEN_MASKS.get(sq1, sq2)
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

#[dynamic]
static BB_BETWEEN_MASKS: SquarePairsToMasks = SquarePairsToMasks::init(calc_between_mask);

fn calc_between_mask(sq1: Square, sq2: Square) -> Bitboard {
    if sq1 == sq2 || !sq1.is_on_same_line_as(sq2) {
        0
    } else {
        let mut mask = 0;
        let direction = unsafe { QueenLikeMoveDirection::lookup_unchecked(sq1, sq2) };
        let mut current = sq1;
        while let Some(next) = current.at(direction) {
            if next == sq2 {
                break;
            }
            mask |= next.mask();
            current = next;
        }
        mask
    }
}

#[cfg(test)]
mod tests {
    use crate::{Bitboard, BitboardUtils, Square};
    use crate::utilities::BitboardDisplay;

    #[test]
    fn test_between_squares() {
        let sq1 = Square::A1;
        let sq2 = Square::H8;
        let mask = Bitboard::between_squares(sq1, sq2);
        mask.print();
    }
}