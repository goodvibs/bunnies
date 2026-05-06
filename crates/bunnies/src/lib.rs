#![feature(const_for)]
#![feature(const_trait_impl)]
#![feature(derive_const)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_param_ty_trait)]
#![feature(const_cmp)]
#![feature(transmutability)]
#![allow(incomplete_features)]
#![allow(unused_features)]

pub mod attacks;
mod bitboard;
mod board;
mod castling;
mod castling_rights;
mod color;
mod colored_piece;
mod context;
mod double_pawn_push_file;
mod fen;
mod file;
mod flank;
mod game_state;
mod insufficient_material;
mod make_move;
mod r#move;
mod move_flag;
mod move_list;
mod movegen;
mod perft;
mod piece;
mod position;
mod rank;
mod san;
mod square;
mod typed_position;
pub mod utilities;
mod validation;
mod zobrist;

pub use bitboard::*;
pub use board::*;
pub use castling_rights::CastlingRights;
pub use color::*;
pub use colored_piece::*;
pub use context::*;
pub use double_pawn_push_file::*;
pub use fen::*;
pub use file::File;
pub use flank::*;
pub use game_state::*;
pub use insufficient_material::*;
pub use r#move::Move;
pub use move_flag::MoveFlag;
pub use move_list::MoveList;
pub use piece::*;
pub use position::Position;
pub use rank::Rank;
pub use square::*;
pub use typed_position::TypedPosition;
pub use validation::*;
pub use zobrist::*;
