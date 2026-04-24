#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(derive_const)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(likely_unlikely)]
#![feature(const_param_ty_trait)]
#![feature(const_cmp)]
#![feature(transmutability)]
#![allow(incomplete_features)]
#![allow(unused_features)]

pub mod attacks;
mod bitboard;
mod castling_rights;
mod color;
mod colored_piece;
mod file;
pub mod flank;
mod r#move;
pub mod pgn;
mod piece;
mod position;
mod rank;
mod square;
pub mod utilities;

pub use bitboard::*;
pub use castling_rights::CastlingRights;
pub use color::*;
pub use colored_piece::*;
pub use file::File;
pub use flank::*;
pub use r#move::*;
pub use piece::*;
pub use position::*;
pub use rank::Rank;
pub use square::*;
