use static_init::dynamic;
use crate::bitboard::Bitboard;
use crate::masks::{ANTIDIAGONALS, DIAGONALS, FILE_A, FILE_H, RANK_1, RANK_8};
use crate::square::Square;

/// Get the precomputed relevant mask for a rook on a given square
pub fn get_rook_relevant_mask(square: Square) -> Bitboard {
    ROOK_RELEVANT_MASKS[square as usize]
}

/// Get the precomputed relevant mask for a bishop on a given square
pub fn get_bishop_relevant_mask(square: Square) -> Bitboard {
    BISHOP_RELEVANT_MASKS[square as usize]
}

/// Precomputed masks for rook relevant squares
#[dynamic]
static ROOK_RELEVANT_MASKS: [Bitboard; 64] = {
    let mut masks = [0; 64];
    for (i, square) in Square::iter_all().enumerate() {
        masks[i] = calc_rook_relevant_mask(*square);
    }
    masks
};

/// Precomputed masks for bishop relevant squares
#[dynamic]
static BISHOP_RELEVANT_MASKS: [Bitboard; 64] = {
    let mut masks = [0; 64];
    for (i, square) in Square::iter_all().enumerate() {
        masks[i] = calc_bishop_relevant_mask(*square);
    }
    masks
};

/// Calculate the relevant mask for a rook on a given square
fn calc_rook_relevant_mask(square: Square) -> Bitboard {
    let file_mask = square.get_file_mask();
    let rank_mask = square.get_rank_mask();
    let mut res = (file_mask | rank_mask) & !square.get_mask();
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
    let square_mask = square.get_mask();
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
    use crate::charboard::print_bb_pretty;

    #[test]
    fn test_calc_rook_relevant_mask() {
        for mask in ROOK_RELEVANT_MASKS.iter() {
            print_bb_pretty(*mask);
            println!();
        }
    }

    #[test]
    fn test_calc_bishop_relevant_mask() {
        for mask in BISHOP_RELEVANT_MASKS.iter() {
            print_bb_pretty(*mask);
            println!();
        }
    }
}