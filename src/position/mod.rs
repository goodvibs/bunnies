//! This module contains game state related code.
//!
//! [`Position<const N: usize>`] holds a fixed-size context stack: `N` slots for the root plus
//! pushed plies (maximum make/unmake depth is `N` including root). Choose `N` at compile time
//! for your deepest `make_move` chain (search, PGN replay, etc.).

mod board;
mod castling;
mod context;
mod fen;
mod insufficient_material;
mod make_move;
mod movegen;
mod perft;
mod r#struct;
mod termination;
mod unmake_move;
mod validation;
mod zobrist;

pub use board::*;
pub use context::*;
pub use fen::*;
pub use r#struct::{Position, PositionError};
pub use termination::*;
pub use zobrist::*;
