//! Stack-allocated list of moves (no heap allocation in the hot path).

use crate::r#move::Move;

/// Fixed-capacity move list stored on the stack, similar to engine-style `MoveList` types.
#[derive(Clone)]
pub struct MoveList<const MAX_MOVES: usize = 256> {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MAX_MOVES: usize> MoveList<MAX_MOVES> {
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
        debug_assert!(self.len < MAX_MOVES, "MoveList overflow: max {}", MAX_MOVES);
        self.moves[self.len] = m;
        self.len += 1;
    }

    #[inline]
    pub fn push_all<const N: usize>(&mut self, moves: [Move; N]) {
        debug_assert!(
            self.len + N <= MAX_MOVES,
            "MoveList overflow: max {}",
            MAX_MOVES
        );
        self.moves[self.len..self.len + N].copy_from_slice(&moves);
        self.len += N;
    }

    #[inline]
    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    #[inline]
    pub fn iter(&self) -> core::slice::Iter<'_, Move> {
        self.as_slice().iter()
    }
}

impl<'a> IntoIterator for &'a MoveList {
    type Item = &'a Move;
    type IntoIter = core::slice::Iter<'a, Move>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
