//! This module contains game state related code.
//!
//! [`Position<const N: usize, const STM: Color>`] holds a fixed-size context stack: `N` slots for the root plus
//! pushed plies (maximum make/unmake depth is `N` including root). `STM` is the side to move
//! ([`Color::White`] or [`Color::Black`]). Choose `N` at compile time
//! for your deepest `make_move` chain (search, PGN replay, etc.).

mod board;
mod castling;
mod context;
mod fen;
mod insufficient_material;
mod legal_gen_kind;
mod make_move;
mod movegen;
mod perft;
mod r#struct;
mod termination;
mod typed_position;
mod unmake_move;
mod validation;
mod zobrist;

pub use board::*;
pub use context::*;
pub use fen::*;
pub use legal_gen_kind::LegalGenKind;
pub use r#struct::{Position, PositionError};
pub use termination::*;
pub use typed_position::TypedPosition;
pub use zobrist::*;
