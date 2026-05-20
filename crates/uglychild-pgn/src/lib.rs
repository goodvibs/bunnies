//! PGN parsing and rendering utilities built on top of `uglychild`.
//!
//! This crate re-exports core chess types from `uglychild` and adds:
//! - [`pgn`] for tokenization, parsing, move-tree construction, and rendering
//! - convenience re-export modules (`position`, `r#move`, `types`) for API parity.
#![feature(const_trait_impl)]
#![feature(derive_const)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_cmp)]
#![allow(incomplete_features)]
#![warn(missing_docs)]

pub use uglychild::{types::*, *};

/// Re-exports of starting-position helpers and position state types.
pub mod position {
    pub use uglychild::{
        logic::fen::INITIAL_FEN,
        types::{Board, Position},
    };
}

/// Re-exports of move primitives used by the PGN parser.
pub mod r#move {
    pub use uglychild::types::{Move, MoveFlag, MoveList};
}

/// Re-exports of shared chess domain types from `uglychild`.
pub mod types {
    pub use uglychild::types::*;
}

/// PGN parser, AST-like game object, tokens, and rendering configuration.
pub mod pgn;

pub use pgn::{PgnError, PgnObject, PgnParser, PgnParsingState, PgnRenderingConfig};
