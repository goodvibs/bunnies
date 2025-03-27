pub mod pgn;
pub mod attacks;
pub mod masks;
pub mod utils;
mod r#move;
mod state;
mod color;
mod square;
mod piece_type;
mod colored_piece_type;

pub use r#move::*;
pub use state::*;
pub use color::*;
pub use square::*;
pub use piece_type::*;
pub use colored_piece_type::*;

/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;