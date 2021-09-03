//! Precalculated large powers for limbs.

#![doc(hidden)]

#[cfg(not(all(target_pointer_width = "64", not(target_arch = "sparc"))))]
pub use crate::large_powers32::*;

#[cfg(all(target_pointer_width = "64", not(target_arch = "sparc")))]
pub use crate::large_powers64::*;
