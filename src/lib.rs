//! Fast, minimal float-parsing algorithm.
//!
//! minimal-lexical has a simple, high-level API with a single
//! exported function: [`parse_float`].
//!
//! [`parse_float`] expects a forward iterator for the integer
//! and fraction digits, as well as a parsed exponent as an [`i32`].
//!
//! For more examples, please see [simple-example](https://github.com/Alexhuszagh/minimal-lexical/blob/master/examples/simple.rs).
//!
//! EXAMPLES
//! --------
//!
//! ```
//! extern crate minimal_lexical;
//!
//! // Let's say we want to parse "1.2345".
//! // First, we need an external parser to extract the integer digits ("1"),
//! // the fraction digits ("2345"), and then parse the exponent to a 32-bit
//! // integer (0).
//! // Warning:
//! // --------
//! //  Please note that leading zeros must be trimmed from the integer,
//! //  and trailing zeros must be trimmed from the fraction. This cannot
//! //  be handled by minimal-lexical, since we accept iterators.
//! let integer = b"1";
//! let fraction = b"2345";
//! let float: f64 = minimal_lexical::parse_float(integer.iter(), fraction.iter(), 0);
//! println!("float={:?}", float);    // 1.235
//! ```
//!
//! [`parse_float`]: fn.parse_float.html
//! [`i32`]: https://doc.rust-lang.org/stable/std/primitive.i32.html

// FEATURES

// Require intrinsics in a no_std context.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(not(feature = "no_alloc"), not(feature = "std")))]
extern crate alloc;

/// Facade around the core features for name mangling.
pub(crate) mod lib {
    #[cfg(feature = "std")]
    pub(crate) use std::*;

    #[cfg(not(feature = "std"))]
    pub(crate) use core::*;

    #[cfg(all(not(feature = "no_alloc"), feature = "std"))]
    pub(crate) use std::vec::Vec;

    #[cfg(all(not(feature = "no_alloc"), not(feature = "std")))]
    pub(crate) use ::alloc::vec::Vec;
}

// MODULES
mod algorithm;
mod bhcomp;
mod bignum;
mod digit;
mod exponent;
mod extended_float;
mod float;
mod large_powers;
mod lemire;
mod math;
mod num;
mod parse;
mod powers;
mod rounding;
mod shift;
mod slice;
mod small_powers;

#[cfg(limb_width_32)]
mod large_powers32;

#[cfg(limb_width_64)]
mod large_powers64;

// API
pub use self::num::Float;
pub use self::parse::parse_float;
