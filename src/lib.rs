/// Facade around the core features for name mangling.
pub(crate) mod lib {
#[cfg(feature = "std")]
pub(crate) use std::*;

#[cfg(not(feature = "std"))]
pub(crate) use core::*;
}

mod algorithm;
mod cached;
mod cached_float80;
mod float;
mod num;
mod rounding;
mod shift;

// API.
pub use self::algorithm::create_float;
pub use self::num::Float;

