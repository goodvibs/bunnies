use crate::Square;
use crate::r#move::MoveType;
use crate::position::TypedPosition;

/// Represents a move in the game.
/// Internally, it is stored as a 16-bit unsigned integer.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    /// format: {6 bit dest}{6 bit src}{4 bit MoveType}
    pub value: u16,
}

impl Move {
    /// Creates a new move. `ty` fully classifies quiet, capture, special, and promotion moves.
    pub const fn new(src: Square, dst: Square, ty: MoveType) -> Move {
        Move {
            value: ((dst as u16) << 10) | ((src as u16) << 4) | (ty as u16),
        }
    }

    /// Gets the destination square of the move.
    pub const fn destination(&self) -> Square {
        let dst_int = (self.value >> 10) as u8;
        Square::from_u8(dst_int)
    }

    /// Gets the source square of the move.
    pub const fn source(&self) -> Square {
        let src_int = ((self.value & 0b0000001111110000) >> 4) as u8;
        Square::from_u8(src_int)
    }

    pub const fn move_type(&self) -> MoveType {
        let t = (self.value & 0b1111) as u8;
        unsafe { MoveType::from_u8(t) }
    }

    pub fn is_capture<const N: usize>(&self, initial_state: &TypedPosition<N>) -> bool {
        let ty = self.move_type();
        if ty.is_capture() {
            return true;
        }
        if ty == MoveType::Castling || ty.is_promotion() {
            return false;
        }
        initial_state.board().is_occupied_at(self.destination())
    }

    /// Returns the UCI (Universal Chess Interface) representation of the move.
    pub fn uci(&self) -> String {
        let promotion_str = if self.move_type().is_promotion() {
            self.move_type()
                .promotion_piece()
                .uppercase_ascii()
                .to_string()
        } else {
            String::new()
        };
        format!(
            "{}{}{}",
            self.source().algebraic(),
            self.destination().algebraic(),
            promotion_str
        )
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.uci())
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Move, MoveType};
    use crate::Piece;
    use crate::Square;

    #[test]
    fn test_move_round_trip_squares_and_type() {
        let src = Square::E2;
        let dst = Square::E4;
        for ty in [
            MoveType::Normal,
            MoveType::DoublePawnPush,
            MoveType::Castling,
            MoveType::NormalCapture,
            MoveType::EnPassant,
            MoveType::PushPromotionToKnight,
            MoveType::PushPromotionToBishop,
            MoveType::PushPromotionToRook,
            MoveType::PushPromotionToQueen,
            MoveType::CapturePromotionToKnight,
            MoveType::CapturePromotionToBishop,
            MoveType::CapturePromotionToRook,
            MoveType::CapturePromotionToQueen,
        ] {
            let mv = Move::new(src, dst, ty);
            assert_eq!(mv.source(), src);
            assert_eq!(mv.destination(), dst);
            assert_eq!(mv.move_type(), ty);
        }
    }

    #[test]
    fn test_move_type_predicates() {
        assert!(!MoveType::Normal.is_capture());
        assert!(!MoveType::DoublePawnPush.is_capture());
        assert!(!MoveType::Castling.is_capture());
        assert!(MoveType::NormalCapture.is_capture());
        assert!(MoveType::EnPassant.is_capture());
        assert!(!MoveType::PushPromotionToQueen.is_capture());
        assert!(MoveType::CapturePromotionToQueen.is_capture());

        assert!(!MoveType::Normal.is_promotion());
        assert!(MoveType::PushPromotionToKnight.is_promotion());
        assert!(MoveType::CapturePromotionToBishop.is_promotion());
    }

    #[test]
    fn test_promotion_piece_round_trip() {
        for piece in Piece::PROMOTION_PIECES {
            let push = MoveType::for_promotion(false, piece);
            assert!(push.is_promotion());
            assert!(!push.is_capture());
            assert_eq!(push.promotion_piece(), piece);

            let cap = MoveType::for_promotion(true, piece);
            assert!(cap.is_promotion());
            assert!(cap.is_capture());
            assert_eq!(cap.promotion_piece(), piece);
        }
    }
}
