//! Fast, minimal float-parsing algorithm.

// FEATURES

// Require intrinsics in a no_std context.
#![cfg_attr(not(feature = "std"), no_std)]

// DEPENDENCIES
#[macro_use]
extern crate cfg_if;

/// Facade around the core features for name mangling.
pub(crate) mod lib {
#[cfg(feature = "std")]
pub(crate) use std::*;

#[cfg(not(feature = "std"))]
pub(crate) use core::*;
}

// MODULES
mod algorithm;
mod bhcomp;
mod bignum;
mod cached;
mod cached_float80;
mod digit;
mod errors;
mod exponent;
mod float;
mod large_powers;
mod math;
mod num;
mod parse;
mod rounding;
mod shift;
mod slice;
mod small_powers;

// API
pub use self::parse::parse_float;
pub use self::num::Float;

