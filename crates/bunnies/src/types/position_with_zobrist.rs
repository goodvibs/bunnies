use super::color::Color;
use super::position::Position;
use super::with_zobrist::WithZobrist;

pub type PositionWithZobrist<const N: usize, const STM: Color> = Position<N, STM, WithZobrist>;
