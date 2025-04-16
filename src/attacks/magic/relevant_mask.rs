use crate::Bitboard;
use crate::Square;
use crate::masks::{DIAGONALS_BL_TO_TR, DIAGONALS_BR_TO_TL, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::utilities::SquaresToMasks;
use static_init::dynamic;

/// Precomputed masks for rook relevant squares
#[dynamic]
pub static ROOK_RELEVANT_MASKS: SquaresToMasks = SquaresToMasks::init(&calc_rook_relevant_mask);

/// Precomputed masks for bishop relevant squares
#[dynamic]
pub static BISHOP_RELEVANT_MASKS: SquaresToMasks = SquaresToMasks::init(&calc_bishop_relevant_mask);

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
    for &diagonal in DIAGONALS_BR_TO_TL.iter() {
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
    }
    for &antidiagonal in DIAGONALS_BL_TO_TR.iter() {
        if antidiagonal & square_mask != 0 {
            res |= antidiagonal;
        }
    }
    res & !square_mask & !(FILE_A | FILE_H | RANK_1 | RANK_8)
}
