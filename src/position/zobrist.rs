//! All Zobrist hashing-related code.

use crate::position::board::Board;
use crate::{Bitboard, PieceType};
use crate::{BitboardUtils, Square};
use rand::Rng;
use static_init::dynamic;

/// A table of random bitboards for each piece type on each square.
#[dynamic]
static ZOBRIST_TABLE: [[Bitboard; 12]; 64] = generate_zobrist_table();

/// Generates a table of random bitboards for each piece type on each square.
fn generate_zobrist_table() -> [[Bitboard; 12]; 64] {
    let mut rng = rand::rng();
    let mut zobrist: [[Bitboard; 12]; 64] = [[0; 12]; 64];
    for i in 0..64 {
        for j in 0..12 {
            zobrist[i][j] = rng.random_range(1..u64::MAX);
        }
    }
    zobrist
}

/// Gets the Zobrist hash for a piece on a square.
pub fn get_piece_zobrist_hash(square: Square, piece_type: PieceType) -> Bitboard {
    ZOBRIST_TABLE[square as usize][piece_type as usize - 1]
}

impl Board {
    /// Calculates the Zobrist hash scratch.
    pub fn calc_zobrist_hash(&self) -> Bitboard {
        let mut hash: Bitboard = 0;
        for piece_type in PieceType::PIECES {
            // skip PieceType::NoPieceType
            let pieces_mask = self.piece_type_masks[piece_type as usize];
            for square in pieces_mask.iter_set_bits_as_squares() {
                hash ^= get_piece_zobrist_hash(square, piece_type);
            }
        }
        hash
    }

    /// Applies the xor of the Zobrist hash of a piece on a square
    pub fn xor_piece_zobrist_hash(&mut self, square: Square, piece_type: PieceType) {
        self.zobrist_hash ^= get_piece_zobrist_hash(square, piece_type)
    }
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
