use crate::Color;
use crate::{Move, MoveList, Position};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TerminalReason {
    Checkmate,
    Stalemate,
    InsufficientMaterial,
    FiftyMoveRule,
    ThreefoldRepetition,
    OtherDraw,
    Win,
    OtherLoss,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ongoing<P>(P);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Terminal<P> {
    position: P,
    reason: TerminalReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameState<P> {
    Ongoing(Ongoing<P>),
    Terminal(Terminal<P>),
}

impl<P> Ongoing<P> {
    #[inline]
    pub fn new(position: P) -> Self {
        Self(position)
    }

    #[inline]
    pub fn into_position(self) -> P {
        self.0
    }

    #[inline]
    pub fn position(&self) -> &P {
        &self.0
    }
}

impl<P> Terminal<P> {
    #[inline]
    pub fn new(position: P, reason: TerminalReason) -> Self {
        Self { position, reason }
    }

    #[inline]
    pub fn reason(&self) -> TerminalReason {
        self.reason
    }

    #[inline]
    pub fn position(&self) -> &P {
        &self.position
    }

    #[inline]
    pub fn into_parts(self) -> (P, TerminalReason) {
        (self.position, self.reason)
    }
}

impl<P> GameState<P> {
    #[inline]
    pub fn from_ongoing(position: P) -> Self {
        Self::Ongoing(Ongoing::new(position))
    }

    #[inline]
    pub fn from_terminal(position: P, reason: TerminalReason) -> Self {
        Self::Terminal(Terminal::new(position, reason))
    }

    #[inline]
    pub fn into_position(self) -> P {
        match self {
            GameState::Ongoing(ongoing) => ongoing.into_position(),
            GameState::Terminal(terminal) => terminal.into_parts().0,
        }
    }

    #[inline]
    pub fn position(&self) -> &P {
        match self {
            GameState::Ongoing(ongoing) => ongoing.position(),
            GameState::Terminal(terminal) => terminal.position(),
        }
    }
}

fn classify_terminal<const N: usize, const STM: Color>(
    pos: &Position<N, STM>,
) -> Option<TerminalReason> {
    if pos.context().halfmove_clock >= 100 {
        return Some(TerminalReason::FiftyMoveRule);
    }

    if pos.board.are_both_sides_insufficient_material(false) {
        return Some(TerminalReason::InsufficientMaterial);
    }

    let mut replies = MoveList::new();
    pos.generate_legal_moves(&mut replies);
    if replies.is_empty() {
        if pos.is_current_side_in_check() {
            Some(TerminalReason::Checkmate)
        } else {
            Some(TerminalReason::Stalemate)
        }
    } else {
        None
    }
}

impl<const N: usize, const STM: Color> Ongoing<Position<N, STM>> {
    #[inline]
    pub fn legal_moves(&self, moves: &mut MoveList) {
        self.0.generate_legal_moves(moves);
    }

    #[inline]
    pub fn play_unchecked(self, mv: Move) -> Ongoing<Position<N, { STM.other() }>> {
        let mut pos = self.0;
        pos.make_move(mv);
        Ongoing(pos.rebrand_stm::<{ STM.other() }>())
    }

    #[inline]
    pub fn play_and_classify(self, mv: Move) -> GameState<Position<N, { STM.other() }>> {
        let next = self.play_unchecked(mv).into_position();
        match classify_terminal(&next) {
            Some(reason) => GameState::from_terminal(next, reason),
            None => GameState::from_ongoing(next),
        }
    }
}
