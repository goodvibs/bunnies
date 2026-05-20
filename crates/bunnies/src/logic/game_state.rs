//! Game state tracking: ongoing vs terminal positions with end reasons.

use crate::types::{Color, Move, MoveList, Position, ZobristPolicy};

/// Reasons why a chess game can end (win, loss, or draw).
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TerminalReason {
    /// Checkmate: current side has no legal moves and is in check.
    Checkmate,
    /// Stalemate: current side has no legal moves but is not in check.
    Stalemate,
    /// Neither side has enough material to checkmate.
    InsufficientMaterial,
    /// 100 half-moves without capture or pawn move.
    FiftyMoveRule,
    /// Position repeated three times (not yet tracked in bunnies).
    ThreefoldRepetition,
    /// Other draw by agreement or rule.
    OtherDraw,
    /// Terminal win (e.g., resignation, timeout with material).
    Win,
    /// Terminal loss (e.g., resignation, timeout with material).
    OtherLoss,
    /// Unclassified terminal state.
    Unknown,
}

/// A position where the game is still in progress.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ongoing<P>(P);

/// A position where the game has ended, with the terminal reason.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Terminal<P> {
    position: P,
    reason: TerminalReason,
}

/// Either an ongoing game or a terminal position.
///
/// Used with the game loop API to track state transitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameState<P> {
    /// Game continues from the wrapped position.
    Ongoing(Ongoing<P>),
    /// Game has ended at the wrapped position.
    Terminal(Terminal<P>),
}

impl<P> Ongoing<P> {
    /// Wraps a position as an ongoing game state.
    #[inline]
    pub fn new(position: P) -> Self {
        Self(position)
    }

    /// Consumes the wrapper and returns the underlying position.
    #[inline]
    pub fn into_position(self) -> P {
        self.0
    }

    /// Borrows the underlying position.
    #[inline]
    pub fn position(&self) -> &P {
        &self.0
    }
}

impl<P> Terminal<P> {
    /// Wraps a terminal position together with the reason the game ended.
    #[inline]
    pub fn new(position: P, reason: TerminalReason) -> Self {
        Self { position, reason }
    }

    /// Returns the terminal reason.
    #[inline]
    pub fn reason(&self) -> TerminalReason {
        self.reason
    }

    /// Borrows the final position.
    #[inline]
    pub fn position(&self) -> &P {
        &self.position
    }

    /// Consumes the wrapper and returns `(position, reason)`.
    #[inline]
    pub fn into_parts(self) -> (P, TerminalReason) {
        (self.position, self.reason)
    }
}

impl<P> GameState<P> {
    /// Creates [`GameState::Ongoing`] from a position.
    #[inline]
    pub fn from_ongoing(position: P) -> Self {
        Self::Ongoing(Ongoing::new(position))
    }

    /// Creates [`GameState::Terminal`] from a position and reason.
    #[inline]
    pub fn from_terminal(position: P, reason: TerminalReason) -> Self {
        Self::Terminal(Terminal::new(position, reason))
    }

    /// Consumes the enum and returns the wrapped position.
    #[inline]
    pub fn into_position(self) -> P {
        match self {
            GameState::Ongoing(ongoing) => ongoing.into_position(),
            GameState::Terminal(terminal) => terminal.into_parts().0,
        }
    }

    /// Borrows the wrapped position regardless of variant.
    #[inline]
    pub fn position(&self) -> &P {
        match self {
            GameState::Ongoing(ongoing) => ongoing.position(),
            GameState::Terminal(terminal) => terminal.position(),
        }
    }
}

fn classify_terminal<const N: usize, const STM: Color, Z: ZobristPolicy>(
    position: &Position<N, STM, Z>,
) -> Option<TerminalReason> {
    if position.context().halfmove_clock >= 100 {
        return Some(TerminalReason::FiftyMoveRule);
    }

    if position
        .board
        .are_both_sides_insufficient_material::<false>()
    {
        return Some(TerminalReason::InsufficientMaterial);
    }

    let mut replies = MoveList::new();
    position.generate_moves(&mut replies);
    if replies.is_empty() {
        if position.is_current_side_in_check() {
            Some(TerminalReason::Checkmate)
        } else {
            Some(TerminalReason::Stalemate)
        }
    } else {
        None
    }
}

impl<const N: usize, const STM: Color, Z: ZobristPolicy> Ongoing<Position<N, STM, Z>> {
    /// Generates legal moves for the current position.
    #[inline]
    pub fn legal_moves(&self, moves: &mut MoveList) {
        self.0.generate_moves(moves);
    }

    /// Applies `move_` and returns the next ongoing state without terminal classification.
    #[inline]
    pub fn play_unchecked(self, move_: Move) -> Ongoing<Position<N, { STM.other() }, Z>> {
        let mut position = self.0;
        position.make_move(move_);
        Ongoing(position.rebrand_stm::<{ STM.other() }>())
    }

    /// Applies `move_`, then classifies whether the resulting position is terminal.
    #[inline]
    pub fn play_and_classify(self, move_: Move) -> GameState<Position<N, { STM.other() }, Z>> {
        let next = self.play_unchecked(move_).into_position();
        match classify_terminal(&next) {
            Some(reason) => GameState::from_terminal(next, reason),
            None => GameState::from_ongoing(next),
        }
    }
}
