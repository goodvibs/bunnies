//! This crate requires a **nightly** Rust toolchain (see repository `rust-toolchain.toml`).
#![feature(const_for)] // Reserved: `for` in `const fn` over arrays needs const `Iterator` (rust#87575).
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![allow(unused_features)] // `const_for` is not yet usable for our `Piece::PIECES` loops; see `Board::piece_at`.

pub mod attacks;
mod bitboard;
mod color;
mod colored_piece;
pub mod flank;
pub mod masks;
mod r#move;
pub mod pgn;
mod piece;
mod position;
mod square;
pub mod utilities;

pub use bitboard::*;
pub use color::*;
pub use colored_piece::*;
pub use flank::*;
pub use r#move::*;
pub use piece::*;
pub use position::*;
pub use square::*;
