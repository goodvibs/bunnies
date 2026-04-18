use crate::Bitboard;
use crate::File;
use crate::Rank;
use crate::Square;
use crate::square::{DIAGONALS_BL_TO_TR, DIAGONALS_BR_TO_TL};
use crate::utilities::SquaresToMasks;

const ROOK_RELEVANT_MASKS_DATA: [Bitboard; 64] = {
    let mut arr = [0u64; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = calc_rook_relevant_mask(unsafe { Square::from_raw(i) });
        i += 1;
    }
    arr
};

const BISHOP_RELEVANT_MASKS_DATA: [Bitboard; 64] = {
    let mut arr = [0u64; 64];
    let mut i = 0u8;
    while i < 64 {
        arr[i as usize] = calc_bishop_relevant_mask(unsafe { Square::from_raw(i) });
        i += 1;
    }
    arr
};

/// Precomputed masks for rook relevant squares
pub static ROOK_RELEVANT_MASKS: SquaresToMasks =
    SquaresToMasks::from_array(ROOK_RELEVANT_MASKS_DATA);

/// Precomputed masks for bishop relevant squares
pub static BISHOP_RELEVANT_MASKS: SquaresToMasks =
    SquaresToMasks::from_array(BISHOP_RELEVANT_MASKS_DATA);

/// Calculate the relevant mask for a rook on a given square
pub const fn calc_rook_relevant_mask(square: Square) -> Bitboard {
    let file_mask = File::from_u8(square.file()).mask();
    let rank_mask = Rank::from_u8(square.rank()).mask();
    let mut res = (file_mask | rank_mask) & !square.mask();
    const EDGE_MASKS: [Bitboard; 4] = [
        File::A.mask(),
        File::H.mask(),
        Rank::One.mask(),
        Rank::Eight.mask(),
    ];
    let mut j = 0;
    while j < 4 {
        let edge_mask = EDGE_MASKS[j];
        if file_mask != edge_mask && rank_mask != edge_mask {
            res &= !edge_mask;
        }
        j += 1;
    }
    res
}

/// Calculate the relevant mask for a bishop on a given square
pub const fn calc_bishop_relevant_mask(square: Square) -> Bitboard {
    let square_mask = square.mask();
    let mut res: Bitboard = 0;
    let mut i = 0;
    while i < 15 {
        let diagonal = DIAGONALS_BR_TO_TL[i];
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
        i += 1;
    }
    i = 0;
    while i < 15 {
        let diagonal = DIAGONALS_BL_TO_TR[i];
        if diagonal & square_mask != 0 {
            res |= diagonal;
        }
        i += 1;
    }
    res & !square_mask
        & !(File::A.mask()
            | File::H.mask()
            | Rank::One.mask()
            | Rank::Eight.mask())
}
