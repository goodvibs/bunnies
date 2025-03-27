use static_init::dynamic;
use crate::utils::Bitboard;
use crate::utils::masks::{ANTIDIAGONALS, DIAGONALS, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::utils::Square;

/// Precomputed masks for rook relevant squares
#[dynamic]
pub static ROOK_RELEVANT_MASKS: PrecomputedMasksForSquares = PrecomputedMasksForSquares::init(&calc_rook_relevant_mask);

/// Precomputed masks for bishop relevant squares
#[dynamic]
pub static BISHOP_RELEVANT_MASKS: PrecomputedMasksForSquares = PrecomputedMasksForSquares::init(&calc_bishop_relevant_mask);

pub struct PrecomputedMasksForSquares {
    masks: [Bitboard; 64]
}

impl PrecomputedMasksForSquares {
    fn init(calc_relevant_mask: &impl Fn(Square) -> Bitboard) -> Self {
        let mut relevant_masks = [0; 64];
        for (i, square) in Square::ALL.into_iter().enumerate() {
            relevant_masks[i] = calc_relevant_mask(square);
        }
        PrecomputedMasksForSquares {
            masks: relevant_masks
        }
    }

    pub fn get(&self, square: Square) -> Bitboard {
        self.masks[square as usize]
    }
}

/// Calculate the relevant mask for a rook on a given square
fn calc_rook_relevant_mask(square: Square) -> Bitboard {
    let file_mask = square.file_mask();
    let rank_mask = square.rank_mask();
    let mut res = (file_mask | rank_mask) & !square.mask();
    let edge_masks = [FILE_A, FILE_H, RANK_1, RANK_8];
    for edge_mask in edge_masks {
        if file_mask != edge_mask && rank_mask != edge_mask {
            res &= !edge_mask;
        }
    }
    res
}

/// Calculate the relevant mask for a bishop on a given square
fn calc_bishop_relevant_mask(square: Square) -> Bitboard {
    let square_mask = square.mask();
    let mut res = 0 as Bitboard;
    for &diagonal in DIAGONALS.iter() {
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
    }
    for &antidiagonal in ANTIDIAGONALS.iter() {
        if antidiagonal & square_mask != 0 {
            res |= antidiagonal;
        }
    }
    res & !square_mask & !(FILE_A | FILE_H | RANK_1 | RANK_8)
}

#[cfg(test)]
mod tests {
    use crate::attacks::magic::relevant_mask::{BISHOP_RELEVANT_MASKS, ROOK_RELEVANT_MASKS};
    use crate::utils::print_bb_pretty;

    #[test]
    fn test_calc_rook_relevant_mask() {
        for mask in ROOK_RELEVANT_MASKS.masks.iter() {
            print_bb_pretty(*mask);
            println!();
        }
    }

    #[test]
    fn test_calc_bishop_relevant_mask() {
        for mask in BISHOP_RELEVANT_MASKS.masks.iter() {
            print_bb_pretty(*mask);
            println!();
        }
    }
}