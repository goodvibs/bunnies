use crate::types::{KnightMoveDirection, QueenLikeMoveDirection};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, Debug)]
#[derive_const(PartialEq)]
pub struct UnifiedMoveDirection {
    pub value: u8,
}

impl UnifiedMoveDirection {
    const NULL_KNIGHT_LIKE: u8 = 0b1111_0000;
    const NULL_QUEEN_LIKE: u8 = 0b0000_1111;

    pub const NULL: UnifiedMoveDirection = UnifiedMoveDirection {
        value: Self::NULL_KNIGHT_LIKE | Self::NULL_QUEEN_LIKE,
    };

    pub const fn from_queen_like(move_direction: QueenLikeMoveDirection) -> UnifiedMoveDirection {
        UnifiedMoveDirection {
            value: Self::NULL_KNIGHT_LIKE | move_direction as u8,
        }
    }

    pub const fn from_knight_like(move_direction: KnightMoveDirection) -> UnifiedMoveDirection {
        UnifiedMoveDirection {
            value: ((move_direction as u8) << 4) | Self::NULL_QUEEN_LIKE,
        }
    }

    /// # Safety
    /// The low nibble of `self.value` must encode either a valid queen-like direction (`0..=7`)
    /// or the queen-like null sentinel (`0b1111`).
    pub const unsafe fn as_queen_like(&self) -> Option<QueenLikeMoveDirection> {
        let value = self.value & Self::NULL_QUEEN_LIKE;
        if value == Self::NULL_QUEEN_LIKE {
            None
        } else {
            Some(unsafe { QueenLikeMoveDirection::from(value) })
        }
    }

    /// # Safety
    /// The low nibble of `self.value` must encode a valid queen-like direction (`0..=7`).
    pub const unsafe fn as_queen_like_unchecked(&self) -> QueenLikeMoveDirection {
        let value = self.value & Self::NULL_QUEEN_LIKE;
        unsafe { QueenLikeMoveDirection::from(value) }
    }

    /// # Safety
    /// The high nibble of `self.value` must encode either a valid knight-like direction (`0..=7`)
    /// or the knight-like null sentinel (`0b1111`).
    pub const unsafe fn as_knight_like(&self) -> Option<KnightMoveDirection> {
        let value = self.value & Self::NULL_KNIGHT_LIKE;
        if value == Self::NULL_KNIGHT_LIKE {
            None
        } else {
            Some(unsafe { KnightMoveDirection::from(value >> 4) })
        }
    }

    /// # Safety
    /// The high nibble of `self.value` must encode a valid knight-like direction (`0..=7`).
    pub const unsafe fn as_knight_like_unchecked(&self) -> KnightMoveDirection {
        let value = self.value & Self::NULL_KNIGHT_LIKE;
        unsafe { KnightMoveDirection::from(value >> 4) }
    }

    pub const fn is_null(&self) -> bool {
        self.value == Self::NULL.value
    }
}

#[cfg(test)]
mod tests {
    use super::{KnightMoveDirection, QueenLikeMoveDirection, UnifiedMoveDirection};
    use crate::utilities::IterableEnum;

    #[test]
    fn test_unified_move_direction() {
        assert!(UnifiedMoveDirection::NULL.is_null());

        assert_eq!(unsafe { UnifiedMoveDirection::NULL.as_knight_like() }, None);
        assert_eq!(unsafe { UnifiedMoveDirection::NULL.as_queen_like() }, None);

        for move_direction in KnightMoveDirection::ALL {
            let unified_move_direction = UnifiedMoveDirection::from_knight_like(move_direction);
            assert_eq!(
                unsafe { unified_move_direction.as_knight_like_unchecked() },
                move_direction
            );
            assert_eq!(unsafe { unified_move_direction.as_queen_like() }, None);
        }
        for move_direction in QueenLikeMoveDirection::ALL {
            let unified_move_direction = UnifiedMoveDirection::from_queen_like(move_direction);
            assert_eq!(
                unsafe { unified_move_direction.as_queen_like_unchecked() },
                move_direction
            );
            assert_eq!(unsafe { unified_move_direction.as_knight_like() }, None);
        }
    }
}
