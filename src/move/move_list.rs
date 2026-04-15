//! Stack-allocated list of moves (no heap allocation in the hot path).

use crate::r#move::Move;

/// Maximum moves in any legal chess position (upper bound ~218; 256 is comfortable).
pub const MAX_MOVES: usize = 256;

/// Fixed-capacity move list stored on the stack, similar to engine-style `MoveList` types.
#[derive(Clone)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    pub const fn new() -> Self {
        Self {
            moves: [Move { value: 0 }; MAX_MOVES],
            len: 0,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn push(&mut self, m: Move) {
        debug_assert!(
            self.len < MAX_MOVES,
            "MoveList overflow: max {}",
            MAX_MOVES
        );
        self.moves[self.len] = m;
        self.len += 1;
    }

    /// Push four promotion moves from the same source/destination.
    #[inline]
    pub fn push_promotions(&mut self, promos: [Move; 4]) {
        for m in promos {
            self.push(m);
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }

    /// Convert to an owned `Vec` for API compatibility.
    pub fn to_vec(&self) -> Vec<Move> {
        self.as_slice().to_vec()
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = core::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
