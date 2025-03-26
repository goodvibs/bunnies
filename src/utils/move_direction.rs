use crate::utils::Square;

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// Represents the direction of a Queen-like move (8 directions).
pub enum QueenLikeMoveDirection {
    Up=0, Down=7,
    UpRight=1, DownLeft=6,
    Right=2, Left=5,
    DownRight=3, UpLeft=4,
}

impl QueenLikeMoveDirection {
    /// An array of all QueenLikeMoveDirections (8 directions).
    pub const ALL: [QueenLikeMoveDirection; 8] = [
        QueenLikeMoveDirection::Up, QueenLikeMoveDirection::Down,
        QueenLikeMoveDirection::UpRight, QueenLikeMoveDirection::DownLeft,
        QueenLikeMoveDirection::Right, QueenLikeMoveDirection::Left,
        QueenLikeMoveDirection::DownRight, QueenLikeMoveDirection::UpLeft
    ];
    
    /// Returns the QueenLikeMoveDirection corresponding to the given value.
    /// # Safety
    /// The value must be in the range 0..=7.
    pub const unsafe fn from(value: u8) -> QueenLikeMoveDirection {
        unsafe { std::mem::transmute::<u8, QueenLikeMoveDirection>(value) }
    }
    
    /// Returns the QueenLikeMoveDirection opposite to the current direction.
    pub const fn opposite(&self) -> QueenLikeMoveDirection {
        unsafe { QueenLikeMoveDirection::from(7u8.wrapping_sub(*self as u8)) }
    }
    
    /// Returns a QueenLikeMoveDirection as calculated from the source and destination squares.
    /// `distance_output` is set to the distance between the source and destination squares.
    /// If the source and destination squares are not in the same line, the behavior is undefined.
    pub const fn calc(src_square: Square, dst_square: Square, distance_output: &mut u8) -> QueenLikeMoveDirection {
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

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// Represents the direction of a Knight move (8 directions).
pub enum KnightMoveDirection {
    TwoUpOneRight=0, TwoDownOneLeft=7,
    TwoRightOneUp=1, TwoLeftOneDown=6,
    TwoRightOneDown=2, TwoLeftOneUp=5,
    TwoDownOneRight=3, TwoUpOneLeft=4,
}

impl KnightMoveDirection {
    /// An array of all KnightMoveDirections (8 directions).
    pub const ALL: [KnightMoveDirection; 8] = [
        KnightMoveDirection::TwoUpOneRight, KnightMoveDirection::TwoDownOneLeft,
        KnightMoveDirection::TwoRightOneUp, KnightMoveDirection::TwoLeftOneDown,
        KnightMoveDirection::TwoRightOneDown, KnightMoveDirection::TwoLeftOneUp,
        KnightMoveDirection::TwoDownOneRight, KnightMoveDirection::TwoUpOneLeft
    ];
    
    /// Returns the KnightMoveDirection opposite to the current direction.
    pub const fn opposite(&self) -> KnightMoveDirection {
        KnightMoveDirection::from(7u8.wrapping_sub(*self as u8))
    }

    /// Returns the KnightMoveDirection corresponding to the given value.
    /// # Safety
    /// The value must be in the range 0..=7.
    pub const fn from(value: u8) -> KnightMoveDirection {
        unsafe { std::mem::transmute::<u8, KnightMoveDirection>(value) }
    }

    /// Returns a KnightMoveDirection as calculated from the source and destination squares.
    /// If the source and destination squares are not a valid Knight move, the behavior is undefined.
    pub const fn calc(src_square: Square, dst_square: Square) -> KnightMoveDirection {
        let value_change = dst_square as i8 - src_square as i8;

        let positive_direction;

        if value_change % 15 == 0 {
            positive_direction = KnightMoveDirection::TwoDownOneLeft;
        } else if value_change % 6 == 0 {
            positive_direction = KnightMoveDirection::TwoLeftOneDown;
        } else if value_change % 17 == 0 {
            positive_direction = KnightMoveDirection::TwoDownOneRight;
        } else {
            positive_direction = KnightMoveDirection::TwoRightOneDown;
        }

        if value_change < 0 {
            positive_direction.opposite()
        } else {
            positive_direction
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
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
                assert_eq!(QueenLikeMoveDirection::calc(square, next_square, &mut distance_found), direction);
                assert_eq!(distance, distance_found);
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
    
    fn test_knight_direction_for_square(square: Square, direction: KnightMoveDirection) {
        let next_square = match direction {
            KnightMoveDirection::TwoUpOneRight => square.up().and_then(|x| x.up_right()),
            KnightMoveDirection::TwoDownOneLeft => square.down().and_then(|x| x.down_left()),
            KnightMoveDirection::TwoRightOneUp => square.right().and_then(|x| x.up_right()),
            KnightMoveDirection::TwoLeftOneDown => square.left().and_then(|x| x.down_left()),
            KnightMoveDirection::TwoRightOneDown => square.right().and_then(|x| x.down_right()),
            KnightMoveDirection::TwoLeftOneUp => square.left().and_then(|x| x.up_left()),
            KnightMoveDirection::TwoDownOneRight => square.down().and_then(|x| x.down_right()),
            KnightMoveDirection::TwoUpOneLeft => square.up().and_then(|x| x.up_left())
        };

        if let Some(next_square) = next_square {
            assert_eq!(KnightMoveDirection::calc(square, next_square), direction);
        }
    }
    
    fn test_all_knight_directions_for_square(square: Square) {
        for direction in KnightMoveDirection::ALL {
            test_knight_direction_for_square(square, direction);
        }
    }
    
    #[test]
    fn test_knight_move_direction() {
        for square in Square::ALL {
            test_all_knight_directions_for_square(square);
        }
    }
}