//! Runtime sum type wrapping [`super::Position`] for API boundaries (FEN, PGN).

use crate::Color;
use crate::position::{FenParseError, Position};

/// Chess position with side to move carried as [`Position`] with const generic `STM` ([`Color::White`] / [`Color::Black`]).
#[derive(Debug)]
pub enum TypedPosition<const N: usize> {
    White(Position<N, { Color::White }>),
    Black(Position<N, { Color::Black }>),
}

impl<const N: usize> Clone for TypedPosition<N> {
    fn clone(&self) -> Self {
        match self {
            TypedPosition::White(p) => TypedPosition::White(p.clone()),
            TypedPosition::Black(p) => TypedPosition::Black(p.clone()),
        }
    }
}

impl<const N: usize> PartialEq for TypedPosition<N> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypedPosition::White(a), TypedPosition::White(b)) => a == b,
            (TypedPosition::Black(a), TypedPosition::Black(b)) => a == b,
            _ => false,
        }
    }
}

impl<const N: usize> Eq for TypedPosition<N> {}

impl<const N: usize> TypedPosition<N> {
    /// Parses a FEN string into a typed position.
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        crate::position::fen::parse_fen_to_typed_position(fen)
    }

    /// Dispatches to the closure corresponding to the compile-time side to move.
    #[inline]
    pub fn with_ref<R, FW, FB>(
        &self,
        white: FW,
        black: FB,
    ) -> R
    where
        FW: FnOnce(&Position<N, { Color::White }>) -> R,
        FB: FnOnce(&Position<N, { Color::Black }>) -> R,
    {
        match self {
            TypedPosition::White(p) => white(p),
            TypedPosition::Black(p) => black(p),
        }
    }

    /// Mutable dispatch to the closure corresponding to the compile-time side to move.
    #[inline]
    pub fn with_mut<R, FW, FB>(
        &mut self,
        white: FW,
        black: FB,
    ) -> R
    where
        FW: FnOnce(&mut Position<N, { Color::White }>) -> R,
        FB: FnOnce(&mut Position<N, { Color::Black }>) -> R,
    {
        match self {
            TypedPosition::White(p) => white(p),
            TypedPosition::Black(p) => black(p),
        }
    }

    /// Consuming dispatch to the closure corresponding to the compile-time side to move.
    #[inline]
    pub fn into_inner<R, FW, FB>(
        self,
        white: FW,
        black: FB,
    ) -> R
    where
        FW: FnOnce(Position<N, { Color::White }>) -> R,
        FB: FnOnce(Position<N, { Color::Black }>) -> R,
    {
        match self {
            TypedPosition::White(p) => white(p),
            TypedPosition::Black(p) => black(p),
        }
    }
}
