//! Runtime sum type wrapping [`super::Position`] for API boundaries (FEN, PGN).

use crate::Color;
use crate::position::{
    Board, FenParseError, GameResult, LegalGenKind, Position, PositionContext,
};
use crate::r#move::{Move, MoveList};

/// Chess position with side to move carried as [`Position`] with const generic `STM` ([`Color::White`] / [`Color::Black`]).
#[derive(Debug)]
pub enum TypedPosition<const N: usize> {
    White(Position<N, { Color::White }>),
    Black(Position<N, { Color::Black }>),
}

impl<const N: usize> Clone for TypedPosition<N> {
    fn clone(&self) -> Self {
        match self {
            TypedPosition::White(p) => TypedPosition::White(p.clone()),
            TypedPosition::Black(p) => TypedPosition::Black(p.clone()),
        }
    }
}

impl<const N: usize> PartialEq for TypedPosition<N> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TypedPosition::White(a), TypedPosition::White(b)) => a == b,
            (TypedPosition::Black(a), TypedPosition::Black(b)) => a == b,
            _ => false,
        }
    }
}

impl<const N: usize> Eq for TypedPosition<N> {}

impl<const N: usize> TypedPosition<N> {
    /// Parses a FEN string into a typed position.
    pub fn from_fen(fen: &str) -> Result<Self, FenParseError> {
        crate::position::fen::parse_fen_to_typed_position(fen)
    }

    #[inline]
    pub const fn side_to_move(&self) -> Color {
        match self {
            TypedPosition::White(_) => Color::White,
            TypedPosition::Black(_) => Color::Black,
        }
    }

    #[inline]
    pub fn board(&self) -> &Board {
        match self {
            TypedPosition::White(p) => &p.board,
            TypedPosition::Black(p) => &p.board,
        }
    }

    #[inline]
    pub fn context(&self) -> &PositionContext {
        match self {
            TypedPosition::White(p) => p.context(),
            TypedPosition::Black(p) => p.context(),
        }
    }

    #[inline]
    pub fn generate_legal_moves(&self, out: &mut MoveList) {
        match self {
            TypedPosition::White(p) => p.generate_legal_moves(out),
            TypedPosition::Black(p) => p.generate_legal_moves(out),
        }
    }

    #[inline]
    pub fn generate_pseudolegal_moves(&self, out: &mut MoveList) {
        match self {
            TypedPosition::White(p) => p.generate_pseudolegal_moves(out),
            TypedPosition::Black(p) => p.generate_pseudolegal_moves(out),
        }
    }

    #[inline]
    pub fn generate_legal_moves_kind(&self, kind: LegalGenKind, out: &mut MoveList) {
        match self {
            TypedPosition::White(p) => p.generate_legal_moves_kind(kind, out),
            TypedPosition::Black(p) => p.generate_legal_moves_kind(kind, out),
        }
    }

    #[inline]
    pub fn is_legal_move(&self, mv: Move) -> bool {
        match self {
            TypedPosition::White(p) => p.is_legal_move(mv),
            TypedPosition::Black(p) => p.is_legal_move(mv),
        }
    }

    /// Applies a legal move and flips the side-to-move marker type.
    ///
    /// This uses [`Position::make_move_in_place`] plus [`Position::rebrand_stm`] instead of
    /// [`Position::make_move`]. With `generic_const_exprs`, calling the generic
    /// `make_move -> Position<N, { STM.other() }>` from this `match` does not satisfy
    /// rustc’s const-generic inference (“unconstrained generic constant”), even when `STM` is fixed
    /// by each arm; the in-place + rebrand sequence matches [`Position::make_move`] exactly.
    pub fn make_move(self, mv: Move) -> Self {
        match self {
            TypedPosition::White(mut p) => {
                p.make_move_in_place(mv);
                TypedPosition::Black(p.rebrand_stm::<{ Color::Black }>())
            }
            TypedPosition::Black(mut p) => {
                p.make_move_in_place(mv);
                TypedPosition::White(p.rebrand_stm::<{ Color::White }>())
            }
        }
    }

    #[inline]
    pub fn get_fullmove(&self) -> u16 {
        match self {
            TypedPosition::White(p) => p.get_fullmove(),
            TypedPosition::Black(p) => p.get_fullmove(),
        }
    }

    #[inline]
    pub fn is_current_side_in_check(&self) -> bool {
        match self {
            TypedPosition::White(p) => p.is_current_side_in_check(),
            TypedPosition::Black(p) => p.is_current_side_in_check(),
        }
    }

    #[inline]
    pub fn is_unequivocally_valid(&self) -> bool {
        match self {
            TypedPosition::White(p) => p.is_unequivocally_valid(),
            TypedPosition::Black(p) => p.is_unequivocally_valid(),
        }
    }

    #[inline]
    pub fn perft(&mut self, depth: u8) -> u64 {
        match self {
            TypedPosition::White(p) => p.perft(depth),
            TypedPosition::Black(p) => p.perft(depth),
        }
    }

    #[inline]
    pub fn result(&self) -> GameResult {
        match self {
            TypedPosition::White(p) => p.result,
            TypedPosition::Black(p) => p.result,
        }
    }

    #[inline]
    pub fn result_mut(&mut self) -> &mut GameResult {
        match self {
            TypedPosition::White(p) => &mut p.result,
            TypedPosition::Black(p) => &mut p.result,
        }
    }
}
