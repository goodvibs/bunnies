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

pub use bunnies::*;

pub mod position {
    pub use bunnies::{Board, Position, INITIAL_FEN};
}

pub mod r#move {
    pub use bunnies::{Move, MoveFlag, MoveList};
}

pub mod pgn;

pub use pgn::*;
