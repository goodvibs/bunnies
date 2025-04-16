use static_init::dynamic;
use crate::Square;
use crate::utilities::SquaresTwoToOneMapping;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct UnifiedMoveDirection {
    pub value: u8
}

impl UnifiedMoveDirection {
    const NULL_KNIGHT_LIKE: u8 = 0b1111_0000;
    const NULL_QUEEN_LIKE: u8 = 0b0000_1111;

    pub const NULL: UnifiedMoveDirection = UnifiedMoveDirection { value: Self::NULL_KNIGHT_LIKE | Self::NULL_QUEEN_LIKE };

    pub const fn from_queen_like(move_direction: QueenLikeMoveDirection) -> UnifiedMoveDirection {
        UnifiedMoveDirection { value: Self::NULL_KNIGHT_LIKE | move_direction as u8 }
    }

    pub const fn from_knight_like(move_direction: KnightMoveDirection) -> UnifiedMoveDirection {
        UnifiedMoveDirection { value: ((move_direction as u8) << 4) | Self::NULL_QUEEN_LIKE }
    }

    pub const unsafe fn as_queen_like(&self) -> Option<QueenLikeMoveDirection> {
        let value = self.value & Self::NULL_QUEEN_LIKE;
        if value == Self::NULL_QUEEN_LIKE {
            None
        } else {
            Some(unsafe { QueenLikeMoveDirection::from(value) })
        }
    }

    pub const unsafe fn as_queen_like_unchecked(&self) -> QueenLikeMoveDirection {
        let value = self.value & Self::NULL_QUEEN_LIKE;
        unsafe { QueenLikeMoveDirection::from(value) }
    }

    pub const unsafe fn as_knight_like(&self) -> Option<KnightMoveDirection> {
        let value = self.value & Self::NULL_KNIGHT_LIKE;
        if value == Self::NULL_KNIGHT_LIKE {
            None
        } else {
            Some(unsafe { KnightMoveDirection::from(value >> 4) })
        }
    }

    pub const unsafe fn as_knight_like_unchecked(&self) -> KnightMoveDirection {
        let value = self.value & Self::NULL_KNIGHT_LIKE;
        unsafe { KnightMoveDirection::from(value >> 4) }
    }

    pub const fn is_null(&self) -> bool {
        self.value == Self::NULL.value
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
    /// An array of all QueenLikeMoveDirections (8 directions).
    pub const ALL: [QueenLikeMoveDirection; 8] = [
        QueenLikeMoveDirection::Up,
        QueenLikeMoveDirection::Down,
        QueenLikeMoveDirection::UpRight,
        QueenLikeMoveDirection::DownLeft,
        QueenLikeMoveDirection::Right,
        QueenLikeMoveDirection::Left,
        QueenLikeMoveDirection::DownRight,
        QueenLikeMoveDirection::UpLeft,
    ];

    /// Returns the QueenLikeMoveDirection corresponding to the given value.
    /// # Safety
    /// The value must be in the range 0..=7.
    pub const unsafe fn from(value: u8) -> QueenLikeMoveDirection {
        unsafe { std::mem::transmute::<u8, QueenLikeMoveDirection>(value) }
    }

    pub fn lookup(src_square: Square, dst_square: Square) -> Option<QueenLikeMoveDirection> {
        unsafe { MOVE_DIRECTION_LOOKUP.get(src_square, dst_square).as_queen_like() }
    }

    pub unsafe fn lookup_unchecked(src_square: Square, dst_square: Square) -> QueenLikeMoveDirection {
        unsafe { MOVE_DIRECTION_LOOKUP.get(src_square, dst_square).as_queen_like_unchecked() }
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

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
/// Represents the direction of a Knight move (8 directions).
pub enum KnightMoveDirection {
    TwoUpOneRight = 0,
    TwoDownOneLeft = 7,
    TwoRightOneUp = 1,
    TwoLeftOneDown = 6,
    TwoRightOneDown = 2,
    TwoLeftOneUp = 5,
    TwoDownOneRight = 3,
    TwoUpOneLeft = 4,
}

impl KnightMoveDirection {
    /// An array of all KnightMoveDirections (8 directions).
    pub const ALL: [KnightMoveDirection; 8] = [
        KnightMoveDirection::TwoUpOneRight,
        KnightMoveDirection::TwoDownOneLeft,
        KnightMoveDirection::TwoRightOneUp,
        KnightMoveDirection::TwoLeftOneDown,
        KnightMoveDirection::TwoRightOneDown,
        KnightMoveDirection::TwoLeftOneUp,
        KnightMoveDirection::TwoDownOneRight,
        KnightMoveDirection::TwoUpOneLeft,
    ];

    /// Returns the KnightMoveDirection opposite to the current direction.
    pub const fn opposite(&self) -> KnightMoveDirection {
        unsafe { KnightMoveDirection::from(7u8.wrapping_sub(*self as u8)) }
    }

    /// Returns the KnightMoveDirection corresponding to the given value.
    /// # Safety
    /// The value must be in the range 0..=7.
    pub const unsafe fn from(value: u8) -> KnightMoveDirection {
        unsafe { std::mem::transmute::<u8, KnightMoveDirection>(value) }
    }

    pub fn lookup(src_square: Square, dst_square: Square) -> Option<KnightMoveDirection> {
        unsafe { MOVE_DIRECTION_LOOKUP.get(src_square, dst_square).as_knight_like() }
    }

    pub unsafe fn lookup_unchecked(src_square: Square, dst_square: Square) -> KnightMoveDirection {
        unsafe { MOVE_DIRECTION_LOOKUP.get(src_square, dst_square).as_knight_like_unchecked() }
    }

    /// Returns a KnightMoveDirection as calculated from the source and destination squares,
    /// or None if the squares are not in a knight move.
    const fn calc(src_square: Square, dst_square: Square) -> Option<KnightMoveDirection> {
        let value_change = dst_square as i8 - src_square as i8;
        
        let is_negative = value_change < 0;
        let value_change_abs = value_change.abs();

        let positive_direction;

        if value_change_abs == 15 {
            positive_direction = KnightMoveDirection::TwoDownOneLeft;
        } else if value_change_abs == 6 {
            positive_direction = KnightMoveDirection::TwoLeftOneDown;
        } else if value_change_abs == 17 {
            positive_direction = KnightMoveDirection::TwoDownOneRight;
        } else if value_change_abs == 10 {
            positive_direction = KnightMoveDirection::TwoRightOneDown;
        } else {
            return None
        }

        if is_negative {
            Some(positive_direction.opposite())
        } else {
            Some(positive_direction)
        }
    }
}

#[dynamic]
static MOVE_DIRECTION_LOOKUP: SquaresTwoToOneMapping<UnifiedMoveDirection> = SquaresTwoToOneMapping::init(
    |src_square, dst_square| {
        if src_square.is_on_same_line_as(dst_square) {
            let direction = QueenLikeMoveDirection::calc(src_square, dst_square, &mut 0);

            UnifiedMoveDirection::from_queen_like(direction)
        }
        else {
            let direction = KnightMoveDirection::calc(src_square, dst_square);

            match direction {
                Some(direction) => UnifiedMoveDirection::from_knight_like(direction),
                None => UnifiedMoveDirection::NULL,
            }
        }
    },
);

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
                assert_eq!(
                    QueenLikeMoveDirection::calc(square, next_square, &mut distance_found),
                    direction
                );
                assert_eq!(distance, distance_found);
                
                assert_eq!(QueenLikeMoveDirection::lookup(square, next_square), Some(direction));
                assert_eq!(QueenLikeMoveDirection::lookup(next_square, square), Some(direction.opposite()));
                assert_eq!(unsafe { QueenLikeMoveDirection::lookup_unchecked(square, next_square) }, direction);
                assert_eq!(unsafe { QueenLikeMoveDirection::lookup_unchecked(next_square, square) }, direction.opposite());
                
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

    fn test_knight_direction_for_square(square: Square, direction: KnightMoveDirection) {
        let next_square = match direction {
            KnightMoveDirection::TwoUpOneRight => square.up().and_then(|x| x.up_right()),
            KnightMoveDirection::TwoDownOneLeft => square.down().and_then(|x| x.down_left()),
            KnightMoveDirection::TwoRightOneUp => square.right().and_then(|x| x.up_right()),
            KnightMoveDirection::TwoLeftOneDown => square.left().and_then(|x| x.down_left()),
            KnightMoveDirection::TwoRightOneDown => square.right().and_then(|x| x.down_right()),
            KnightMoveDirection::TwoLeftOneUp => square.left().and_then(|x| x.up_left()),
            KnightMoveDirection::TwoDownOneRight => square.down().and_then(|x| x.down_right()),
            KnightMoveDirection::TwoUpOneLeft => square.up().and_then(|x| x.up_left()),
        };

        if let Some(next_square) = next_square {
            assert_eq!(KnightMoveDirection::calc(square, next_square), Some(direction));
            
            assert_eq!(KnightMoveDirection::lookup(square, next_square), Some(direction));
            assert_eq!(KnightMoveDirection::lookup(next_square, square), Some(direction.opposite()));
            assert_eq!(unsafe { KnightMoveDirection::lookup_unchecked(square, next_square) }, direction);
            assert_eq!(unsafe { KnightMoveDirection::lookup_unchecked(next_square, square) }, direction.opposite());
            
            assert_eq!(QueenLikeMoveDirection::lookup(square, next_square), None);
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

    #[test]
    fn test_unified_move_direction() {
        assert!(UnifiedMoveDirection::NULL.is_null());

        assert_eq!(unsafe { UnifiedMoveDirection::NULL.as_knight_like() }, None);
        assert_eq!(unsafe { UnifiedMoveDirection::NULL.as_queen_like() }, None);

        for move_direction in KnightMoveDirection::ALL {
            let unified_move_direction = UnifiedMoveDirection::from_knight_like(move_direction);
            assert_eq!(unsafe { unified_move_direction.as_knight_like_unchecked() }, move_direction);
            assert_eq!(unsafe { unified_move_direction.as_queen_like() }, None);
        }
        for move_direction in QueenLikeMoveDirection::ALL {
            let unified_move_direction = UnifiedMoveDirection::from_queen_like(move_direction);
            assert_eq!(unsafe { unified_move_direction.as_queen_like_unchecked() }, move_direction);
            assert_eq!(unsafe { unified_move_direction.as_knight_like() }, None);
        }
    }
}
