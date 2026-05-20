use crate::{
    impl_u8_conversions,
    types::{Array, Square},
    utilities::IterableEnum,
};

#[repr(u8)]
#[derive(Copy, Clone, Eq, Debug)]
#[derive_const(PartialEq)]
/// Represents the direction of a Queen-like move (8 directions).
pub enum QueenLikeMoveDirection {
    Up = 0,
    Down = 7,
    UpRight = 1,
    DownLeft = 6,
    Right = 2,
    Left = 5,
    DownRight = 3,
    UpLeft = 4,
}

impl QueenLikeMoveDirection {
    /// Returns the QueenLikeMoveDirection corresponding to the given value.
    /// # Safety
    /// The value must be in the range 0..=7.
    pub const unsafe fn from(value: u8) -> QueenLikeMoveDirection {
        unsafe { std::mem::transmute::<u8, QueenLikeMoveDirection>(value) }
    }

    pub fn lookup(src_square: Square, dst_square: Square) -> Option<QueenLikeMoveDirection> {
        unsafe {
            super::MOVE_DIRECTION_LOOKUP[src_square as usize][dst_square as usize].as_queen_like()
        }
    }

    /// # Safety
    /// `src_square` and `dst_square` must be on the same line (rank/file/diagonal).
    pub unsafe fn lookup_unchecked(
        src_square: Square,
        dst_square: Square,
    ) -> QueenLikeMoveDirection {
        unsafe {
            super::MOVE_DIRECTION_LOOKUP[src_square as usize][dst_square as usize]
                .as_queen_like_unchecked()
        }
    }

    /// Returns the QueenLikeMoveDirection opposite to the current direction.
    pub const fn opposite(&self) -> QueenLikeMoveDirection {
        unsafe { QueenLikeMoveDirection::from(7u8.wrapping_sub(*self as u8)) }
    }

    /// Returns a QueenLikeMoveDirection as calculated from the source and destination squares.
    /// `distance_output` is set to the distance between the source and destination squares.
    /// If the source and destination squares are not in the same line, the behavior is undefined.
    pub const fn calc(
        src_square: Square,
        dst_square: Square,
        distance_output: &mut u8,
    ) -> QueenLikeMoveDirection {
        let value_change = dst_square as i8 - src_square as i8;

        let positive_direction;
        let displacement;

        if value_change % 8 == 0 {
            positive_direction = QueenLikeMoveDirection::Down;
            displacement = value_change / 8;
        } else if value_change % 9 == 0 {
            positive_direction = QueenLikeMoveDirection::DownRight;
            displacement = value_change / 9;
        } else if src_square.rank() == dst_square.rank() {
            positive_direction = QueenLikeMoveDirection::Right;
            displacement = value_change;
        } else {
            positive_direction = QueenLikeMoveDirection::DownLeft;
            displacement = value_change / 7;
        }

        if value_change < 0 {
            *distance_output = -displacement as u8;
            positive_direction.opposite()
        } else {
            *distance_output = displacement as u8;
            positive_direction
        }
    }
}

impl const IterableEnum<8> for QueenLikeMoveDirection {
    const ALL: Array<QueenLikeMoveDirection, 8> = Array([
        QueenLikeMoveDirection::Up,
        QueenLikeMoveDirection::Down,
        QueenLikeMoveDirection::UpRight,
        QueenLikeMoveDirection::DownLeft,
        QueenLikeMoveDirection::Right,
        QueenLikeMoveDirection::Left,
        QueenLikeMoveDirection::DownRight,
        QueenLikeMoveDirection::UpLeft,
    ]);
}

impl_u8_conversions!(QueenLikeMoveDirection, 8);

#[cfg(test)]
mod tests {
    use super::{QueenLikeMoveDirection, Square};
    use crate::{types::KnightMoveDirection, utilities::IterableEnum};

    fn test_queen_direction_for_square(square: Square, direction: QueenLikeMoveDirection) {
        let mut current_square = square;
        let mut distance = 0;
        loop {
            let next_square = match direction {
                QueenLikeMoveDirection::Up => current_square.up(),
                QueenLikeMoveDirection::Down => current_square.down(),
                QueenLikeMoveDirection::Right => current_square.right(),
                QueenLikeMoveDirection::Left => current_square.left(),
                QueenLikeMoveDirection::UpRight => current_square.up_right(),
                QueenLikeMoveDirection::DownLeft => current_square.down_left(),
                QueenLikeMoveDirection::DownRight => current_square.down_right(),
                QueenLikeMoveDirection::UpLeft => current_square.up_left(),
            };

            if let Some(next_square) = next_square {
                distance += 1;
                let mut distance_found = 0;
                assert_eq!(
                    QueenLikeMoveDirection::calc(square, next_square, &mut distance_found),
                    direction
                );
                assert_eq!(distance, distance_found);

                assert_eq!(
                    QueenLikeMoveDirection::lookup(square, next_square),
                    Some(direction)
                );
                assert_eq!(
                    QueenLikeMoveDirection::lookup(next_square, square),
                    Some(direction.opposite())
                );
                assert_eq!(
                    unsafe { QueenLikeMoveDirection::lookup_unchecked(square, next_square) },
                    direction
                );
                assert_eq!(
                    unsafe { QueenLikeMoveDirection::lookup_unchecked(next_square, square) },
                    direction.opposite()
                );

                assert_eq!(KnightMoveDirection::lookup(square, next_square), None);

                current_square = next_square;
            } else {
                break;
            }
        }
    }

    fn test_all_queen_directions_for_square(square: Square) {
        for direction in QueenLikeMoveDirection::ALL {
            test_queen_direction_for_square(square, direction);
        }
    }

    #[test]
    fn test_queen_move_direction() {
        for square in Square::ALL {
            test_all_queen_directions_for_square(square);
        }
    }
}
