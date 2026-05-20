//! Compact 16-bit chess move encoding.

use super::{board::Board, move_flag::MoveFlag, piece::Piece, square::Square};

/// A chess move encoded in 16 bits.
///
/// Bit layout: `TTTTTT FFFFFF PP GG` where:
/// - `T` (6 bits): destination square index (0-63)
/// - `F` (6 bits): source square index (0-63)
/// - `P` (2 bits): promotion piece offset (Piece value minus 2, for Knight/Queen/Rook/Bishop)
/// - `G` (2 bits): move flag (normal, promotion, en passant, castling)
///
/// Use [`Move::new`] to construct, and [`Move::from`], [`Move::to`], [`Move::promotion`], [`Move::flag`]
/// to decompose. The default value (0) is a valid move from A8 to A8 with null flag.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Move {
    /// The raw 16-bit encoded move value.
    pub value: u16,
}

impl Move {
    /// The default promotion value for a move.
    pub const DEFAULT_PROMOTION_VALUE: Piece = Piece::Rook;

    /// Creates a new move.
    pub const fn new(from: Square, to: Square, promotion: Piece, flag: MoveFlag) -> Move {
        debug_assert!(
            !matches!(promotion, Piece::King | Piece::Pawn),
            "Invalid promotion piece type"
        );
        Move {
            value: ((to as u16) << 10)
                | ((from as u16) << 4)
                | ((promotion as u16 - 2) << 2)
                | flag as u16,
        }
    }

    /// Creates a new move with the default promotion value.
    pub const fn new_non_promotion(from: Square, to: Square, flag: MoveFlag) -> Move {
        Move::new(from, to, Move::DEFAULT_PROMOTION_VALUE, flag)
    }

    /// Creates a promotion move (flag set to [`MoveFlag::Promotion`]).
    ///
    /// `promotion` must be one of [`Piece::PROMOTION_PIECES`].
    pub const fn new_promotion(from: Square, to: Square, promotion: Piece) -> Move {
        Move::new(from, to, promotion, MoveFlag::Promotion)
    }

    /// Gets the target square of the move.
    pub const fn to(&self) -> Square {
        let to_int = (self.value >> 10) as u8;
        unsafe { Square::try_from(to_int).unwrap_unchecked() }
    }

    /// Gets the origin square of the move.
    pub const fn from(&self) -> Square {
        let from_int = ((self.value & 0b0000001111110000) >> 4) as u8;
        unsafe { Square::try_from(from_int).unwrap_unchecked() }
    }

    /// Gets the promotion piece type of the move.
    pub const fn promotion(&self) -> Piece {
        let promotion_int = ((self.value & 0b0000000000001100) >> 2) as u8;
        unsafe { Piece::from(promotion_int + 2) }
    }

    /// Gets the flag of the move.
    pub const fn flag(&self) -> MoveFlag {
        let flag_int = (self.value & 0b0000000000000011) as u8;
        unsafe { MoveFlag::from(flag_int) }
    }

    /// Returns `true` if this move captures a piece on `board`.
    ///
    /// Normal moves and promotions capture when destination is occupied.
    /// En passant is always treated as capture; castling never captures.
    pub const fn is_capture_on_board(&self, board: &Board) -> bool {
        match self.flag() {
            MoveFlag::NormalMove | MoveFlag::Promotion => board.is_occupied_at(self.to()),
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
            self.from().algebraic(),
            self.to().algebraic(),
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
    use crate::{
        types::{Piece, Square},
        utilities::IterableEnum,
    };

    #[test]
    fn test_move() {
        for to in Square::ALL {
            for from in Square::ALL {
                for promotion_piece in Piece::PROMOTION_PIECES {
                    for flag_int in 0..4 {
                        let flag = unsafe { MoveFlag::from(flag_int) };

                        let move_ = Move::new(from, to, promotion_piece, flag);
                        assert_eq!(move_.to(), to);
                        assert_eq!(move_.from(), from);
                        assert_eq!(move_.promotion(), promotion_piece);
                        assert_eq!(move_.flag(), flag);
                    }
                }
            }
        }
    }
}
