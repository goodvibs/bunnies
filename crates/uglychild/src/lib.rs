//! `uglychild` is a const-friendly chess engine core focused on fast move generation,
//! compact board representations, and strongly typed game state transitions.
//!
//! The crate is organized into:
//! - [`types`] for core domain models (`Position`, `Board`, `Move`, `Square`, etc.)
//! - [`logic`] for parsing, SAN/FEN helpers, legality checks, and other algorithms.
//!
//! Most consumers will interact with [`types::Position`] plus move generation APIs.
#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_iter)]
#![feature(const_default)]
#![feature(const_precise_live_drops)]
#![feature(const_index)]
#![feature(const_slice_make_iter)]
#![feature(const_result_unwrap_unchecked)]
#![feature(derive_const)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_cmp)]
#![allow(incomplete_features)]

/// High-level chess rules and notation logic built on top of core types.
pub mod logic;
/// Core chess data structures and low-level operations.
pub mod types;

mod utilities;
