//! This module contains game state related code.

mod board;
mod castling;
mod context;
mod fen;
mod r#struct;
mod insufficient_material;
mod make_move;
mod movegen;
mod perft;
mod termination;
mod unmake_move;
mod validation;
mod zobrist;

pub use board::*;
pub use context::*;
pub use fen::*;
pub use r#struct::*;
pub use termination::*;
pub use zobrist::*;
