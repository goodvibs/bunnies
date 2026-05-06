use crate::{Bitboard, Square};

/// Lookup table keyed by square (64 entries).
#[derive(Clone, Copy)]
pub struct SquaresToMasks {
    data: [Bitboard; 64],
}

impl SquaresToMasks {
    pub const fn from_array(data: [Bitboard; 64]) -> Self {
        Self { data }
    }

    #[inline]
    pub const fn get(&self, square: Square) -> Bitboard {
        self.data[square as usize]
    }
}

/// Lookup table keyed by `(sq1, sq2)` in row-major order: `index = sq1 * 64 + sq2`.
#[derive(Clone, Copy)]
pub struct SquarePairsToMasks {
    data: [Bitboard; 64 * 64],
}

impl SquarePairsToMasks {
    pub const fn from_array(data: [Bitboard; 64 * 64]) -> Self {
        Self { data }
    }

    #[inline]
    pub const fn get(&self, sq1: Square, sq2: Square) -> Bitboard {
        self.data[(sq1 as usize) * 64 + (sq2 as usize)]
    }
}

/// Pair-keyed table (e.g. bool or small `Copy` aggregates).
#[derive(Clone, Copy)]
pub struct SquaresTwoToOneMapping<T: Copy> {
    data: [T; 64 * 64],
}

impl<T: Copy> SquaresTwoToOneMapping<T> {
    pub const fn from_array(data: [T; 64 * 64]) -> Self {
        Self { data }
    }

    #[inline]
    pub const fn get(&self, sq1: Square, sq2: Square) -> T {
        self.data[(sq1 as usize) * 64 + (sq2 as usize)]
    }
}
