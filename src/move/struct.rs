use crate::Piece;
use crate::Square;
use crate::r#move::MoveFlag;
use crate::position::Position;

/// Represents a move in the game.
/// Internally, it is stored as a 16-bit unsigned integer.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    /// format: {6 bit dest}{6 bit src}{4 bit MoveFlag value}
    pub value: u16,
}

impl Move {
    /// Creates a new move.
    pub const fn new(src: Square, dst: Square, flag: MoveFlag) -> Move {
        Move {
            value: ((dst as u16) << 10)
                | ((src as u16) << 4)
                | flag as u16,
        }
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

    /// Gets the flag of the move.
    pub const fn flag(&self) -> MoveFlag {
        let flag_int = (self.value & 0b0000000000001111) as u8;
        unsafe { MoveFlag::from(flag_int) }
    }
    
    pub const fn promotion(&self) -> Piece {
        match self.flag() {
            MoveFlag::PromotionToKnight => Piece::Knight,
            MoveFlag::PromotionToBishop => Piece::Bishop,
            MoveFlag::PromotionToRook => Piece::Rook,
            MoveFlag::PromotionToQueen => Piece::Queen,
            _ => Piece::Null
        }
    }

    pub fn is_capture(&self, initial_state: &Position) -> bool {
        match self.flag() {
            MoveFlag::NormalPawnPush | MoveFlag::PawnDoublePush | MoveFlag::ShortCastling | MoveFlag::LongCastling => false,
            MoveFlag::NormalPawnCapture | MoveFlag::EnPassant => true,
            _ => {
                initial_state.board.is_occupied_at(self.destination())
            }
        }
    }

    /// Returns the UCI (Universal Chess Interface) representation of the move.
    pub fn uci(&self) -> String {
        let promotion = self.promotion();
        let promotion_str = if promotion != Piece::Null {
            promotion.uppercase_ascii().to_string()
        } else {
            "".to_string()
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
    use crate::Square;

    #[test]
    fn test_move() {
        for dst_square in Square::ALL {
            for src_square in Square::ALL {
                for flag_int in MoveFlag::Null as u8..=MoveFlag::LongCastling as u8 {
                    let flag = unsafe { MoveFlag::from(flag_int) };

                    let mv = Move::new(src_square, dst_square, flag);
                    assert_eq!(mv.destination(), dst_square);
                    assert_eq!(mv.source(), src_square);
                    assert_eq!(mv.flag(), flag);
                }
            }
        }
    }
}
