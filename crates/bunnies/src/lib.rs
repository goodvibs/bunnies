#![feature(const_trait_impl)]
#![feature(const_convert)]
#![feature(const_iter)]
#![feature(const_default)]
#![feature(const_precise_live_drops)]
#![feature(const_index)]
#![feature(const_slice_make_iter)]
#![feature(derive_const)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(const_cmp)]
#![allow(incomplete_features)]

// Public modules
pub mod attacks;
pub mod io;
pub mod logic;
pub mod types;

// Private modules
mod utils;
