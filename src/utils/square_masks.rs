use crate::{Bitboard, Square};

/// Stores 64 masks for the 64 squares on the board, initialized using a provided initializer function
pub struct SquareMasks {
    masks: [Bitboard; 64]
}

impl SquareMasks {
    /// Initializes the precomputed masks for squares using the provided mask calculation function
    pub fn init(calc_mask: &impl Fn(Square) -> Bitboard) -> Self {
        let mut masks = [0; 64];
        for (i, square) in Square::ALL.into_iter().enumerate() {
            masks[i] = calc_mask(square);
        }
        SquareMasks {
            masks
        }
    }

    /// Returns the mask for a given square
    pub fn get(&self, square: Square) -> Bitboard {
        self.masks[square as usize]
    }
}