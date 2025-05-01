use crate::Piece;

/// Enum representing the different types of moves that can be made in a game of chess.
/// Used in the Move struct.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveFlag {
    Null = 0,
    NormalPawnPush = 1,
    PawnDoublePush = 2,
    NormalPawnCapture = 3,
    EnPassant = 4,
    PromotionToKnight = 5,
    PromotionToBishop = 6,
    PromotionToRook = 7,
    PromotionToQueen = 8,
    KnightMove = 9,
    BishopMove = 10,
    RookMove = 11,
    QueenMove = 12,
    KingMove = 13,
    ShortCastling = 14,
    LongCastling = 15
}

impl MoveFlag {
    /// Converts a u8 value to a MoveFlag.
    pub const unsafe fn from(value: u8) -> MoveFlag {
        unsafe { std::mem::transmute::<u8, MoveFlag>(value) }
    }
    
    pub const fn is_null(&self) -> bool {
        *self as u8 == Self::Null as u8
    }
    
    pub const fn is_promotion(&self) -> bool {
        (Self::PromotionToKnight as u8) <= (*self as u8) && (*self as u8) <= (Self::PromotionToQueen as u8)
    }
    
    pub const fn is_pawn_push(&self) -> bool {
        *self as u8 == Self::NormalPawnPush as u8 || *self as u8 == Self::PawnDoublePush as u8
    }
    
    pub const fn is_pawn_move(&self) -> bool {
        (Self::NormalPawnPush as u8) <= (*self as u8) && (*self as u8) <= (Self::EnPassant as u8)
    }

    pub const fn is_castling(&self) -> bool {
        *self as u8 == Self::ShortCastling as u8 || *self as u8 == Self::LongCastling as u8
    }
    
    pub const fn is_guaranteed_non_capture(&self) -> bool {
        self.is_castling() || self.is_pawn_push()
    }

    pub const fn is_guaranteed_capture(&self) -> bool {
        matches!(self, Self::EnPassant | Self::NormalPawnCapture)
    }
    
    pub const fn moved_piece(&self) -> Piece {
        if *self as u8 == Self::Null as u8 {
            Piece::Null
        } else if *self as u8 <= Self::PromotionToQueen as u8 {
            Piece::Pawn
        } else if *self as u8 >= Self::KingMove as u8 {
            Piece::King
        } else {
            let val_from_knight = *self as u8 - MoveFlag::KnightMove as u8;
            unsafe { Piece::from(val_from_knight + Piece::Knight as u8 ) }
        }
    }
}
