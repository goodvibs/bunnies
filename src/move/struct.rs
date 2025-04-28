use crate::PieceType;
use crate::Square;
use crate::r#move::MoveFlag;
use crate::position::Position;

/// Represents a move in the game.
/// Internally, it is stored as a 16-bit unsigned integer.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    /// format: {6 bit dest}{6 bit src}{2 bit promotion PieceType value minus 2}{2 bit MoveFlag value}
    pub value: u16,
}

impl Move {
    /// The default promotion value for a move.
    pub const DEFAULT_PROMOTION_VALUE: PieceType = PieceType::Rook;

    /// Creates a new move.
    pub fn new(src: Square, dst: Square, promotion: PieceType, flag: MoveFlag) -> Move {
        assert!(
            promotion != PieceType::King && promotion != PieceType::Pawn,
            "Invalid promotion piece type"
        );
        Move {
            value: ((dst as u16) << 10)
                | ((src as u16) << 4)
                | ((promotion as u16 - 2) << 2)
                | flag as u16,
        }
    }

    /// Creates a new move with the default promotion value.
    pub fn new_non_promotion(src: Square, dst: Square, flag: MoveFlag) -> Move {
        Move::new(src, dst, Move::DEFAULT_PROMOTION_VALUE, flag)
    }

    pub fn new_promotion(src: Square, dst: Square, promotion: PieceType) -> Move {
        Move::new(src, dst, promotion, MoveFlag::Promotion)
    }

    /// Gets the destination square of the move.
    pub const fn destination(&self) -> Square {
        let dst_int = (self.value >> 10) as u8;
        unsafe { Square::from(dst_int) }
    }

    /// Gets the source square of the move.
    pub const fn source(&self) -> Square {
        let src_int = ((self.value & 0b0000001111110000) >> 4) as u8;
        unsafe { Square::from(src_int) }
    }

    /// Gets the promotion piece type of the move.
    pub const fn promotion(&self) -> PieceType {
        let promotion_int = ((self.value & 0b0000000000001100) >> 2) as u8;
        unsafe { PieceType::from(promotion_int + 2) }
    }

    /// Gets the flag of the move.
    pub const fn flag(&self) -> MoveFlag {
        let flag_int = (self.value & 0b0000000000000011) as u8;
        unsafe { MoveFlag::from(flag_int) }
    }

    pub fn is_capture(&self, initial_state: &Position) -> bool {
        match self.flag() {
            MoveFlag::NormalMove | MoveFlag::Promotion => {
                initial_state.board.is_occupied_at(self.destination())
            }
            MoveFlag::EnPassant => true,
            MoveFlag::Castling => false,
        }
    }

    /// Returns the UCI (Universal Chess Interface) representation of the move.
    pub fn uci(&self) -> String {
        let promotion_str = match self.flag() {
            MoveFlag::Promotion => self.promotion().uppercase_ascii().to_string(),
            _ => "".to_string(),
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
    use super::{Move, MoveFlag};
    use crate::PieceType;
    use crate::Square;

    #[test]
    fn test_move() {
        for dst_square in Square::ALL {
            for src_square in Square::ALL {
                for promotion_piece in PieceType::PROMOTION_PIECES {
                    for flag_int in 0..4 {
                        let flag = unsafe { MoveFlag::from(flag_int) };

                        let mv = Move::new(src_square, dst_square, promotion_piece, flag);
                        assert_eq!(mv.destination(), dst_square);
                        assert_eq!(mv.source(), src_square);
                        assert_eq!(mv.promotion(), promotion_piece);
                        assert_eq!(mv.flag(), flag);
                    }
                }
            }
        }
    }
}
