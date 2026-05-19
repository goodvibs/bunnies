use super::color::Color;
use super::position::Position;
use super::without_zobrist::WithoutZobrist;

pub type PositionWithoutZobrist<const N: usize, const STM: Color> =
    Position<N, STM, WithoutZobrist>;
