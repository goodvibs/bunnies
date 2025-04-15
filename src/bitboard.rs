/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;

pub fn bb_between(sq1: u64, sq2: u64) -> Bitboard {
    let mut bb = 0;
    let mut sq = sq1;
    while sq != sq2 {
        bb |= sq;
        sq = (sq + 1) & !sq1;
    }
    bb
}