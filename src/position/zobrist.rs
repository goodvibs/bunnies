//! All Zobrist hashing-related code.

use crate::position::board::Board;
use crate::{Bitboard, Piece};
use crate::{BitboardUtils, Square};

/// Fixed pseudo-random table (const-evaluated). Keys are stable across runs and builds.
const fn zobrist_table() -> [[Bitboard; 12]; 64] {
    let mut zobrist = [[0u64; 12]; 64];
    let mut x: u64 = 0x243F_6A88_85A3_08D3;
    let mut i = 0;
    while i < 64 {
        let mut j = 0;
        while j < 12 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let v = x.rotate_left(((i * 13 + j * 7) % 64) as u32);
            zobrist[i][j] = if v == 0 { 1 } else { v };
            j += 1;
        }
        i += 1;
    }
    zobrist
}

static ZOBRIST_TABLE: [[Bitboard; 12]; 64] = zobrist_table();

/// Gets the Zobrist hash for a piece on a square.
pub fn get_piece_zobrist_hash(square: Square, piece_type: Piece) -> Bitboard {
    ZOBRIST_TABLE[square as usize][piece_type as usize - 1]
}

/// Piece-placement Zobrist hash (no side-to-move or castling/ep).
pub fn calc_zobrist_hash(board: &Board) -> Bitboard {
    let mut hash: Bitboard = 0;
    for piece_type in Piece::PIECES {
        let pieces_mask = board.piece_mask(piece_type);
        for square in pieces_mask.iter_set_bits_as_squares() {
            hash ^= get_piece_zobrist_hash(square, piece_type);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_zobrist_hash() {
        // todo
    }

    #[test]
    fn test_increment_position_count() {
        // todo
    }

    #[test]
    fn test_decrement_position_count() {
        // todo
    }
}
