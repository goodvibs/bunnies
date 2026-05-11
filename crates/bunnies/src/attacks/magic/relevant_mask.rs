use crate::Bitboard;
use crate::File;
use crate::Rank;
use crate::Square;
use crate::square::{DIAGONALS_BL_TO_TR, DIAGONALS_BR_TO_TL};
use crate::utilities::Array;
use crate::utilities::SquaresToMasks;

const ROOK_RELEVANT_MASKS_DATA: Array<Bitboard, 64> = Array({
    let mut arr = [0u64; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = calc_rook_relevant_mask(Square::from_u8(i));
        i += 1;
    }
    arr
});

const BISHOP_RELEVANT_MASKS_DATA: Array<Bitboard, 64> = Array({
    let mut arr = [0u64; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = calc_bishop_relevant_mask(Square::from_u8(i));
        i += 1;
    }
    arr
});

/// Precomputed masks for rook relevant squares
pub static ROOK_RELEVANT_MASKS: SquaresToMasks =
    SquaresToMasks::from_array(ROOK_RELEVANT_MASKS_DATA.0);

/// Precomputed masks for bishop relevant squares
pub static BISHOP_RELEVANT_MASKS: SquaresToMasks =
    SquaresToMasks::from_array(BISHOP_RELEVANT_MASKS_DATA.0);

/// Calculate the relevant mask for a rook on a given square
pub const fn calc_rook_relevant_mask(square: Square) -> Bitboard {
    let file_mask = square.file().mask();
    let rank_mask = square.rank().mask();
    let mut res = (file_mask | rank_mask) & !square.mask();
    const EDGE_MASKS: Array<Bitboard, 4> = Array([
        File::A.mask(),
        File::H.mask(),
        Rank::One.mask(),
        Rank::Eight.mask(),
    ]);
    for edge_mask in EDGE_MASKS {
        if file_mask != edge_mask && rank_mask != edge_mask {
            res &= !edge_mask;
        }
    }
    res
}

/// Calculate the relevant mask for a bishop on a given square
pub const fn calc_bishop_relevant_mask(square: Square) -> Bitboard {
    let square_mask = square.mask();
    let mut res: Bitboard = 0;
    for diagonal in DIAGONALS_BR_TO_TL {
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
    }
    for diagonal in DIAGONALS_BL_TO_TR {
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
    }
    res & !square_mask & !(File::A.mask() | File::H.mask() | Rank::One.mask() | Rank::Eight.mask())
}
