use crate::Piece;

/// Enum representing the different types of moves that can be made in a game of chess.
/// Used in the Move struct.
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, std::marker::ConstParamTy)]
#[rustfmt::skip]
pub enum MoveType {
    Normal                   = 0b0000,
    DoublePawnPush           = 0b0001,
    Castling                 = 0b0010,

    NormalCapture            = 0b0100,
    EnPassant                = 0b0101,

    PushPromotionToKnight    = 0b1000,
    PushPromotionToBishop    = 0b1001,
    PushPromotionToRook      = 0b1010,
    PushPromotionToQueen     = 0b1011,

    CapturePromotionToKnight = 0b1100,
    CapturePromotionToBishop = 0b1101,
    CapturePromotionToRook   = 0b1110,
    CapturePromotionToQueen  = 0b1111,
}

impl MoveType {
    /// Converts the low 4 bits of `value` to a [`MoveType`] (packed in [`crate::Move`]).
    pub const unsafe fn from_u8(value: u8) -> MoveType {
        debug_assert!(value <= 0b1111, "MoveType must fit in 4 bits");
        unsafe { std::mem::transmute::<u8, MoveType>(value & 0b1111) }
    }

    /// `piece` must be Knight, Bishop, Rook, or Queen.
    pub const fn for_promotion(is_capture: bool, piece: Piece) -> Self {
        let low: u8 = match piece {
            Piece::Knight => 0,
            Piece::Bishop => 1,
            Piece::Rook => 2,
            Piece::Queen => 3,
            _ => 0,
        };
        let high: u8 = if is_capture { 0b1100 } else { 0b1000 };
        unsafe { Self::from_u8(high | low) }
    }

    pub const fn is_capture(self) -> bool {
        self as u8 & 0b0100 != 0
    }

    pub const fn is_promotion(self) -> bool {
        self as u8 & 0b1000 != 0
    }

    pub const fn moved_piece(self) -> Piece {
        match self {
            Self::Normal => Piece::Null,
            Self::Castling => Piece::King,
            Self::NormalCapture => Piece::Null,
            _ => Piece::Pawn,
        }
    }

    const fn promotion_bits(self) -> u8 {
        self as u8 & 0b0011
    }

    /// Only valid when [`Self::is_promotion`] is true.
    pub const fn promotion_piece(self) -> Piece {
        unsafe { Piece::from(self.promotion_bits() + Piece::Knight as u8) }
    }
}
