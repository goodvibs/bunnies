//! This module contains game state related code.

mod board;
mod castling;
mod game_context;
mod fen;
mod game_state;
mod insufficient_material;
mod make_move;
mod movegen;
mod perft;
mod termination;
mod unmake_move;
mod validation;
mod zobrist;
mod attacks_by_color;

pub use board::*;
pub use game_context::*;
pub use fen::*;
pub use game_state::*;
pub use termination::*;
pub use zobrist::*;
pub use attacks_by_color::*;
