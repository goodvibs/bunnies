/// Which subset of legal moves to generate (split for quiescence search and ordering).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalGenKind {
    All,
    Captures,
    Quiets,
}
