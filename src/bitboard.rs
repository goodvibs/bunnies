use crate::Square;
use crate::square::same_line;
use crate::utilities::{
    BitCombinationsIterator, MaskBitsIterator, MaskSquaresIterator, QueenLikeMoveDirection,
};

/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;

/// Const-friendly ray geometry between squares (also a supertrait of [`BitboardUtils`]).
pub const trait ConstBitboardGeometry {
    /// Returns the mask of squares **strictly between** the two squares (endpoints excluded).
    /// On a rank, file, or diagonal; otherwise returns zero.
    fn between(sq1: Square, sq2: Square) -> Bitboard;

    /// Returns the mask of squares that form a line connecting two squares, extending to the
    /// edges of the board.
    /// This includes orthogonal and diagonal lines. If none exist, zero is returned.
    fn edge_to_edge_ray(sq1: Square, sq2: Square) -> Bitboard;
}

pub trait BitboardUtils: ConstBitboardGeometry {
    /// Returns an iterator that generates the set bits of the bitboard.
    fn iter_set_bits_as_masks(self) -> MaskBitsIterator;

    /// Returns an iterator that generates the squares of the bitboard.
    fn iter_set_bits_as_squares(self) -> MaskSquaresIterator;

    /// Returns an iterator that generates all possible set bit combinations of the bitboard.
    fn iter_bit_combinations(self) -> BitCombinationsIterator;
}

impl const ConstBitboardGeometry for Bitboard {
    fn between(sq1: Square, sq2: Square) -> Bitboard {
        MASK_BETWEEN_DATA[(sq1 as usize) * 64 + (sq2 as usize)]
    }

    fn edge_to_edge_ray(sq1: Square, sq2: Square) -> Bitboard {
        EDGE_TO_EDGE_RAY_DATA[(sq1 as usize) * 64 + (sq2 as usize)]
    }
}

impl BitboardUtils for Bitboard {
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

const fn calc_between(sq1: Square, sq2: Square) -> Bitboard {
    if sq1 as u8 == sq2 as u8 || !same_line(sq1, sq2) {
        0
    } else {
        let mut dist = 0;
        let direction = QueenLikeMoveDirection::calc(sq1, sq2, &mut dist);
        let mut mask: Bitboard = 0;
        let mut current = sq1;
        loop {
            match current.neighbor_in_direction(direction) {
                None => break,
                Some(next) => {
                    if next as u8 == sq2 as u8 {
                        break;
                    }
                    mask |= next.mask();
                    current = next;
                }
            }
        }
        mask
    }
}

const fn calc_edge_to_edge_ray(sq1: Square, sq2: Square) -> Bitboard {
    if sq1 as u8 == sq2 as u8 || !same_line(sq1, sq2) {
        0
    } else {
        let mut dist = 0;
        let direction = QueenLikeMoveDirection::calc(sq1, sq2, &mut dist);
        let mut mask = sq1.mask();
        let mut current = sq1;
        loop {
            match current.neighbor_in_direction(direction) {
                None => break,
                Some(next) => {
                    mask |= next.mask();
                    current = next;
                }
            }
        }
        current = sq1;
        loop {
            match current.neighbor_in_direction(direction.opposite()) {
                None => break,
                Some(next) => {
                    mask |= next.mask();
                    current = next;
                }
            }
        }
        mask
    }
}

const MASK_BETWEEN_DATA: [Bitboard; 64 * 64] = {
    let mut arr = [0u64; 64 * 64];
    let mut i = 0usize;
    while i < 64 * 64 {
        let sq1 = Square::from_u8((i / 64) as u8);
        let sq2 = Square::from_u8((i % 64) as u8);
        arr[i] = calc_between(sq1, sq2);
        i += 1;
    }
    arr
};

const EDGE_TO_EDGE_RAY_DATA: [Bitboard; 64 * 64] = {
    let mut arr = [0u64; 64 * 64];
    let mut i = 0usize;
    while i < 64 * 64 {
        let sq1 = Square::from_u8((i / 64) as u8);
        let sq2 = Square::from_u8((i % 64) as u8);
        arr[i] = calc_edge_to_edge_ray(sq1, sq2);
        i += 1;
    }
    arr
};

#[cfg(test)]
mod tests {
    use crate::utilities::BitboardDisplay;
    use crate::{Bitboard, ConstBitboardGeometry, Square};

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
