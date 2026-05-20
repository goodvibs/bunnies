//! High-level chess algorithms built on top of core [`crate::types`] primitives.
//!
//! This module contains move generation/execution, notation helpers (FEN/SAN),
//! terminal-state classification, attack tables, and validation utilities.

/// Attack generation helpers (manual and magic-bitboard based).
pub mod attacks;
/// Castling-rights updates and castling-specific helpers.
pub mod castling;
/// FEN parsing into strongly typed positions.
pub mod fen;
/// Ongoing/terminal game-state wrappers and classification.
pub mod game_state;
/// Insufficient-material detection routines.
pub mod insufficient_material;
/// In-place `make_move`/`unmake_move` transition logic.
pub mod make_move;
/// Legal move generation and counting APIs on [`crate::types::Position`].
pub mod move_generation;
/// Perft node-count benchmarking helpers.
pub mod perft;
/// Standard Algebraic Notation rendering.
pub mod san;
/// Position consistency and legality validation checks.
pub mod validation;
/// Zobrist hashing keys and position hash calculation.
pub mod zobrist_hash;
