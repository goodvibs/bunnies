pub mod attacks;
mod color;
mod colored_piece_type;
pub mod masks;
mod r#move;
pub mod pgn;
mod piece_type;
mod square;
mod state;
pub mod utilities;

pub use color::*;
pub use colored_piece_type::*;
pub use r#move::*;
pub use piece_type::*;
pub use square::*;
pub use state::*;

/// A type alias for a bitboard. A bitboard is a 64-bit unsigned integer that represents an aspect of board state.
/// Each bit represents a square on the board, with the most significant bit representing A8 and the least significant bit representing H1.
pub type Bitboard = u64;
