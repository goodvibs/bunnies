//! Runtime sum type wrapping [`super::Position`] for API boundaries (FEN, PGN).

use crate::Color;
use crate::{FenParseError, Position, WithZobrist, ZobristPolicy};

/// Chess position with side to move carried as [`Position`] with const generic `STM` ([`Color::White`] / [`Color::Black`]).
#[derive(Debug)]
pub enum TypedPosition<const N: usize, Z: ZobristPolicy = WithZobrist> {
    White(Position<N, { Color::White }, Z>),
    Black(Position<N, { Color::Black }, Z>),
}

impl<const N: usize, Z: ZobristPolicy> Clone for TypedPosition<N, Z> {
    fn clone(&self) -> Self {
        match self {
            TypedPosition::White(p) => TypedPosition::White(p.clone()),
            TypedPosition::Black(p) => TypedPosition::Black(p.clone()),
        }
    }
}

impl<const N: usize, Z: ZobristPolicy> PartialEq for TypedPosition<N, Z> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypedPosition::White(a), TypedPosition::White(b)) => a == b,
            (TypedPosition::Black(a), TypedPosition::Black(b)) => a == b,
            _ => false,
        }
    }
}

impl<const N: usize, Z: ZobristPolicy> Eq for TypedPosition<N, Z> {}

impl<const N: usize, Z: ZobristPolicy> TypedPosition<N, Z> {
    /// Parses a FEN string into a typed position.
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        crate::parse_fen_to_typed_position(fen)
    }

    /// Dispatches to the closure corresponding to the compile-time side to move.
    #[inline]
    pub fn with_ref<R, FW, FB>(&self, white: FW, black: FB) -> R
    where
        FW: FnOnce(&Position<N, { Color::White }, Z>) -> R,
        FB: FnOnce(&Position<N, { Color::Black }, Z>) -> R,
    {
        match self {
            TypedPosition::White(p) => white(p),
            TypedPosition::Black(p) => black(p),
        }
    }

    /// Mutable dispatch to the closure corresponding to the compile-time side to move.
    #[inline]
    pub fn with_mut<R, FW, FB>(&mut self, white: FW, black: FB) -> R
    where
        FW: FnOnce(&mut Position<N, { Color::White }, Z>) -> R,
        FB: FnOnce(&mut Position<N, { Color::Black }, Z>) -> R,
    {
        match self {
            TypedPosition::White(p) => white(p),
            TypedPosition::Black(p) => black(p),
        }
    }

    /// Consuming dispatch to the closure corresponding to the compile-time side to move.
    #[inline]
    pub fn into_inner<R, FW, FB>(self, white: FW, black: FB) -> R
    where
        FW: FnOnce(Position<N, { Color::White }, Z>) -> R,
        FB: FnOnce(Position<N, { Color::Black }, Z>) -> R,
    {
        match self {
            TypedPosition::White(p) => white(p),
            TypedPosition::Black(p) => black(p),
        }
    }
}
