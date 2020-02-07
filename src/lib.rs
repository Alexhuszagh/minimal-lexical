/// Facade around the core features for name mangling.
pub(crate) mod lib {
#[cfg(feature = "std")]
pub(crate) use std::*;

#[cfg(not(feature = "std"))]
pub(crate) use core::*;
}

mod cached;
mod cached_float80;
mod float;
mod num;
mod parse;
mod rounding;
mod shift;

// API.
pub use self::num::Float;
pub use self::parse::parse_float;

