//! PGN parsing and rendering utilities built on top of `bunnies`.
//!
//! This crate re-exports core chess types from `bunnies` and adds:
//! - [`pgn`] for tokenization, parsing, move-tree construction, and rendering
//! - convenience re-export modules (`position`, `r#move`, `types`) for API parity.
#![feature(const_trait_impl)]
#![feature(derive_const)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_cmp)]
#![allow(incomplete_features)]
#![warn(missing_docs)]

pub use bunnies::{types::*, *};

/// Re-exports of starting-position helpers and position state types.
pub mod position {
    pub use bunnies::{
        logic::fen::INITIAL_FEN,
        types::{Board, Position},
    };
}

/// Re-exports of move primitives used by the PGN parser.
pub mod r#move {
    pub use bunnies::types::{Move, MoveFlag, MoveList};
}

/// Re-exports of shared chess domain types from `bunnies`.
pub mod types {
    pub use bunnies::types::*;
}

/// PGN parser, AST-like game object, tokens, and rendering configuration.
pub mod pgn;

pub use pgn::{PgnError, PgnObject, PgnParser, PgnParsingState, PgnRenderingConfig};
