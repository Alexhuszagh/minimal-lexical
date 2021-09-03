//! Precalculated large powers for limbs.

#![doc(hidden)]

#[cfg(limb_width_32)]
pub use crate::large_powers32::*;

#[cfg(limb_width_64)]
pub use crate::large_powers64::*;
