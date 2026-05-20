//! Move classification flags for special move handling.

/// Classification of special chess move types encoded in the low 2 bits of a [`Move`](crate::types::Move).
#[repr(u8)]
#[derive(Clone, Copy, Eq, Debug)]
#[derive_const(PartialEq)]
pub enum MoveFlag {
    /// Ordinary move (including captures that aren't special).
    NormalMove = 0,
    /// Pawn promotion (promotion piece encoded in move bits).
    Promotion = 1,
    /// En passant capture.
    EnPassant = 2,
    /// King castling move (kingside or queenside determined by destination).
    Castling = 3,
}

impl MoveFlag {
    /// Creates a `MoveFlag` from its numeric value.
    ///
    /// # Safety
    /// `value` must be in range `0..4`. Values outside this range are undefined behavior.
    pub const unsafe fn from(value: u8) -> MoveFlag {
        debug_assert!(value < 4, "Invalid MoveFlag value");
        unsafe { std::mem::transmute::<u8, MoveFlag>(value) }
    }

    /// Human-readable label for debugging.
    pub const fn to_readable(&self) -> &str {
        match self {
            MoveFlag::NormalMove => "",
            MoveFlag::Promotion => "[P to ?]",
            MoveFlag::EnPassant => "[e.p.]",
            MoveFlag::Castling => "[castling]",
        }
    }
}

impl From<u8> for MoveFlag {
    fn from(value: u8) -> MoveFlag {
        unsafe { MoveFlag::from(value) }
    }
}
