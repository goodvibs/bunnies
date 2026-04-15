//! Compile-time side-to-move markers for [`super::Position`].

use crate::Color;

mod sealed {
    pub trait Sealed {}
}

/// Marker for positions where White is to move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WhiteToMove;

/// Marker for positions where Black is to move.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlackToMove;

impl sealed::Sealed for WhiteToMove {}
impl sealed::Sealed for BlackToMove {}

/// Sealed trait tying a zero-sized marker to the side to move and its opposite marker.
pub trait SideState: sealed::Sealed + Copy + core::fmt::Debug + 'static {
    const STM: Color;
    type Other: SideState;
}

impl SideState for WhiteToMove {
    const STM: Color = Color::White;
    type Other = BlackToMove;
}

impl SideState for BlackToMove {
    const STM: Color = Color::Black;
    type Other = WhiteToMove;
}
