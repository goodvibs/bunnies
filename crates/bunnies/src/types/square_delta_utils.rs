use super::color::Color;
use super::square_delta::SquareDelta;

mod private {
    pub const trait Sealed {}
    impl Sealed for super::SquareDelta {}
}

pub const trait SquareDeltaUtils: private::Sealed {
    const UP: SquareDelta;
    const DOWN: SquareDelta;
    const LEFT: SquareDelta;
    const RIGHT: SquareDelta;

    const UP_LEFT: SquareDelta;
    const DOWN_LEFT: SquareDelta;
    const UP_RIGHT: SquareDelta;
    const DOWN_RIGHT: SquareDelta;

    fn for_perspective(self, color: Color) -> SquareDelta;
}

impl const SquareDeltaUtils for SquareDelta {
    const UP: SquareDelta = -8;
    const DOWN: SquareDelta = 8;
    const LEFT: SquareDelta = -1;
    const RIGHT: SquareDelta = 1;

    const UP_LEFT: SquareDelta = SquareDelta::UP + SquareDelta::LEFT;
    const DOWN_LEFT: SquareDelta = SquareDelta::DOWN + SquareDelta::LEFT;
    const UP_RIGHT: SquareDelta = SquareDelta::UP + SquareDelta::RIGHT;
    const DOWN_RIGHT: SquareDelta = SquareDelta::DOWN + SquareDelta::RIGHT;

    fn for_perspective(self, color: Color) -> SquareDelta {
        self * match color {
            Color::White => 1,
            Color::Black => -1,
        }
    }
}
