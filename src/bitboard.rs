use crate::Square;
use crate::utilities::{
    BitCombinationsIterator, MaskBitsIterator, MaskSquaresIterator, QueenLikeMoveDirection,
    SquarePairsToMasks,
};
use static_init::dynamic;

/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;

pub trait BitboardUtils {
    /// Returns the mask of squares between two squares, inclusive/exclusive??.
    /// This includes orthogonal and diagonal lines. If none exist, zero is returned.
    fn between(sq1: Square, sq2: Square) -> Bitboard;

    /// Returns the mask of squares that form a line connecting two squares, extending to the
    /// edges of the board.
    /// This includes orthogonal and diagonal lines. If none exist, zero is returned.
    fn edge_to_edge_ray(sq1: Square, sq2: Square) -> Bitboard;

    /// Returns an iterator that generates the set bits of the bitboard.
    fn iter_set_bits_as_masks(self) -> MaskBitsIterator;

    /// Returns an iterator that generates the squares of the bitboard.
    fn iter_set_bits_as_squares(self) -> MaskSquaresIterator;

    /// Returns an iterator that generates all possible set bit combinations of the bitboard.
    fn iter_bit_combinations(self) -> BitCombinationsIterator;
}

impl BitboardUtils for Bitboard {
    fn between(sq1: Square, sq2: Square) -> Bitboard {
        MASK_BETWEEN_LOOKUP.get(sq1, sq2)
    }

    fn edge_to_edge_ray(sq1: Square, sq2: Square) -> Bitboard {
        EDGE_TO_EDGE_RAY_LOOKUP.get(sq1, sq2)
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
static MASK_BETWEEN_LOOKUP: SquarePairsToMasks = SquarePairsToMasks::init(calc_between);

fn calc_between(sq1: Square, sq2: Square) -> Bitboard {
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

#[dynamic]
static EDGE_TO_EDGE_RAY_LOOKUP: SquarePairsToMasks =
    SquarePairsToMasks::init(calc_edge_to_edge_ray);

fn calc_edge_to_edge_ray(sq1: Square, sq2: Square) -> Bitboard {
    if sq1 == sq2 || !sq1.is_on_same_line_as(sq2) {
        0
    } else {
        let mut mask = 0;
        let direction = unsafe { QueenLikeMoveDirection::lookup_unchecked(sq1, sq2) };

        let mut current = sq1;
        while let Some(next) = current.at(direction) {
            mask |= next.mask();
            current = next;
        }

        current = sq1;
        while let Some(next) = current.at(direction.opposite()) {
            mask |= next.mask();
            current = next;
        }

        mask |= sq1.mask();

        mask
    }
}

#[cfg(test)]
mod tests {
    use crate::utilities::BitboardDisplay;
    use crate::{Bitboard, BitboardUtils, Square};

    #[test]
    fn test_between() {
        let sq1 = Square::A1;
        let sq2 = Square::H8;
        let mask = Bitboard::between(sq1, sq2);
        mask.print();
    }

    #[test]
    fn test_edge_to_edge_ray() {
        let sq1 = Square::A1;
        let sq2 = Square::H8;
        let mask = Bitboard::edge_to_edge_ray(sq1, sq2);
        mask.print();

        println!();

        let sq1 = Square::B6;
        let sq2 = Square::E3;
        let mask = Bitboard::edge_to_edge_ray(sq1, sq2);
        mask.print();

        println!();

        let sq1 = Square::G3;
        let sq2 = Square::G7;
        let mask = Bitboard::edge_to_edge_ray(sq1, sq2);
        mask.print();

        println!();

        let sq1 = Square::B7;
        let sq2 = Square::B3;
        let mask = Bitboard::edge_to_edge_ray(sq1, sq2);
        mask.print();
    }
}
