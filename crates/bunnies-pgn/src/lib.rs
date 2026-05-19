#![feature(const_trait_impl)]
#![feature(derive_const)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_cmp)]
#![allow(incomplete_features)]

pub use bunnies::types::*;
pub use bunnies::*;

pub mod position {
    pub use bunnies::io::fen::INITIAL_FEN;
    pub use bunnies::types::{Board, Position};
}

pub mod r#move {
    pub use bunnies::types::{Move, MoveFlag, MoveList};
}

pub mod types {
    pub use bunnies::types::*;
}

pub mod pgn;

pub use pgn::{PgnError, PgnObject, PgnParser, PgnParsingState, PgnRenderingConfig};
