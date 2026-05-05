//! This module contains game state related code.
//!
//! [`Position<const N: usize, const STM: Color>`] holds a fixed-size context stack: `N` slots for the root plus
//! pushed plies (maximum make/unmake depth is `N` including root). `STM` is the side to move
//! ([`Color::White`] or [`Color::Black`]). Choose `N` at compile time for your deepest `make_move`
//! chain (search, PGN replay, etc.). Exceeding `N` is a **contract violation** (debug panic; UB in release).

mod board;
mod castling;
mod context;
mod double_pawn_push_file;
mod fen;
mod game_state;
mod insufficient_material;
mod legal_gen_kind;
mod make_move;
mod movegen;
mod perft;
mod r#struct;
mod termination;
mod typed_position;
mod validation;
mod zobrist;

pub use board::*;
pub use context::*;
pub use double_pawn_push_file::*;
pub use fen::*;
pub use game_state::*;
pub use legal_gen_kind::LegalGenKind;
pub use r#struct::Position;
pub use termination::*;
pub use typed_position::TypedPosition;
pub use zobrist::*;
