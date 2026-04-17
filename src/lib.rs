//! This crate requires a **nightly** Rust toolchain (see repository `rust-toolchain.toml`).
#![feature(const_for)] // Reserved: `for` in `const fn` over arrays needs const `Iterator` (rust#87575).
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]
#![allow(unused_features)] // Reserved for `const_for` over arrays when const `Iterator` stabilizes.

pub mod attacks;
mod bitboard;
mod castling_rights;
mod color;
mod colored_piece;
mod file;
pub mod flank;
mod r#move;
mod rank;
pub mod pgn;
mod piece;
mod position;
mod square;
pub mod utilities;

pub use bitboard::*;
pub use castling_rights::CastlingRights;
pub use color::*;
pub use colored_piece::*;
pub use file::File;
pub use flank::*;
pub use rank::Rank;
pub use r#move::*;
pub use piece::*;
pub use position::*;
pub use square::*;
