//! This module contains game state related code.

mod board;
mod context;
mod termination;
mod make_move;
mod movegen;
mod unmake_move;
mod zobrist;
mod fen;
mod r#struct;
mod validation;
mod castling;
mod insufficient_material;
mod perft;

pub use r#struct::*;
pub use board::*;
pub use context::*;
pub use termination::*;
pub use zobrist::*;
pub use fen::*;
