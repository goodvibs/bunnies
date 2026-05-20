use crate::{
    impl_u8_conversions,
    types::{Array, Square},
    utilities::IterableEnum,
};

#[repr(u8)]
#[derive(Copy, Clone, Eq, Debug)]
#[derive_const(PartialEq)]
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
        unsafe {
            super::MOVE_DIRECTION_LOOKUP[src_square as usize][dst_square as usize].as_knight_like()
        }
    }

    /// # Safety
    /// `src_square` and `dst_square` must form a legal knight displacement.
    pub unsafe fn lookup_unchecked(src_square: Square, dst_square: Square) -> KnightMoveDirection {
        unsafe {
            super::MOVE_DIRECTION_LOOKUP[src_square as usize][dst_square as usize]
                .as_knight_like_unchecked()
        }
    }

    /// Returns a KnightMoveDirection as calculated from the source and destination squares,
    /// or None if the squares are not in a knight move.
    pub(crate) const fn calc(
        src_square: Square,
        dst_square: Square,
    ) -> Option<KnightMoveDirection> {
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
            return None;
        }

        if is_negative {
            Some(positive_direction.opposite())
        } else {
            Some(positive_direction)
        }
    }
}

impl const IterableEnum<8> for KnightMoveDirection {
    const ALL: Array<KnightMoveDirection, 8> = Array([
        KnightMoveDirection::TwoUpOneRight,
        KnightMoveDirection::TwoDownOneLeft,
        KnightMoveDirection::TwoRightOneUp,
        KnightMoveDirection::TwoLeftOneDown,
        KnightMoveDirection::TwoRightOneDown,
        KnightMoveDirection::TwoLeftOneUp,
        KnightMoveDirection::TwoDownOneRight,
        KnightMoveDirection::TwoUpOneLeft,
    ]);
}

impl_u8_conversions!(KnightMoveDirection, 8);

#[cfg(test)]
mod tests {
    use super::{KnightMoveDirection, Square};
    use crate::{types::QueenLikeMoveDirection, utilities::IterableEnum};

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
            assert_eq!(
                KnightMoveDirection::calc(square, next_square),
                Some(direction)
            );

            assert_eq!(
                KnightMoveDirection::lookup(square, next_square),
                Some(direction)
            );
            assert_eq!(
                KnightMoveDirection::lookup(next_square, square),
                Some(direction.opposite())
            );
            assert_eq!(
                unsafe { KnightMoveDirection::lookup_unchecked(square, next_square) },
                direction
            );
            assert_eq!(
                unsafe { KnightMoveDirection::lookup_unchecked(next_square, square) },
                direction.opposite()
            );

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
}
