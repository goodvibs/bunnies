//! Contains the Termination enum and its implementation.

/// Represents the different ways a game can end.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum GameResult {
    // Ongoing game
    None,
    // Win
    Win,
    // Loss
    Checkmate,
    OtherLoss,
    // Draw
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule,
    OtherDraw,
    // Unknown result
    Unknown,
}

impl GameResult {
    pub fn is_none(self) -> bool {
        matches!(self, GameResult::None)
    }

    pub fn is_win(self) -> bool {
        matches!(self, GameResult::Win)
    }

    pub fn is_loss(self) -> bool {
        matches!(self, GameResult::Checkmate | GameResult::OtherLoss)
    }

    pub fn is_draw(self) -> bool {
        matches!(
            self,
            GameResult::Stalemate
                | GameResult::InsufficientMaterial
                | GameResult::ThreefoldRepetition
                | GameResult::FiftyMoveRule
                | GameResult::OtherDraw
        )
    }

    pub fn is_unknown(self) -> bool {
        matches!(self, GameResult::Unknown)
    }
}
