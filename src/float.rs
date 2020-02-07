// FLOAT TYPE

use super::convert::*;
use super::num::*;
use super::rounding::*;
use super::shift::*;

/// Extended precision floating-point type.
///
/// Private implementation, exposed only for testing purposes.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExtendedFloat {
    /// Mantissa for the extended-precision float.
    pub mant: u64,
    /// Binary exponent for the extended-precision float.
    pub exp: i32,
}

impl ExtendedFloat {
    // PROPERTIES

    // OPERATIONS

    /// Multiply two normalized extended-precision floats, as if by `a*b`.
    ///
    /// The precision is maximal when the numbers are normalized, however,
    /// decent precision will occur as long as both values have high bits
    /// set. The result is not normalized.
    ///
    /// Algorithm:
    ///     1. Non-signed multiplication of mantissas (requires 2x as many bits as input).
    ///     2. Normalization of the result (not done here).
    ///     3. Addition of exponents.
    pub fn mul(&self, b: &ExtendedFloat) -> ExtendedFloat {
        // Logic check, values must be decently normalized prior to multiplication.
        debug_assert!((self.mant & u64::HIMASK != 0) && (b.mant & u64::HIMASK != 0));

        // Extract high-and-low masks.
        let ah = self.mant >> u64::HALF;
        let al = self.mant & u64::LOMASK;
        let bh = b.mant >> u64::HALF;
        let bl = b.mant & u64::LOMASK;

        // Get our products
        let ah_bl = ah * bl;
        let al_bh = al * bh;
        let al_bl = al * bl;
        let ah_bh = ah * bh;

        let mut tmp = (ah_bl & u64::LOMASK) + (al_bh & u64::LOMASK) + (al_bl >> u64::HALF);
        // round up
        tmp += 1 << (u64::HALF-1);

        ExtendedFloat {
            mant: ah_bh + (ah_bl >> u64::HALF) + (al_bh >> u64::HALF) + (tmp >> u64::HALF),
            exp: self.exp + b.exp + u64::FULL
        }
    }

    /// Multiply in-place, as if by `a*b`.
    ///
    /// The result is not normalized.
    pub fn imul(&mut self, b: &ExtendedFloat) {
        *self = self.mul(b);
    }

    // NORMALIZE

    /// Normalize float-point number.
    ///
    /// Shift the mantissa so the number of leading zeros is 0, or the value
    /// itself is 0.
    ///
    /// Get the number of bytes shifted.
    pub fn normalize(&mut self) -> u32 {
        // Note:
        // Using the cltz intrinsic via leading_zeros is way faster (~10x)
        // than shifting 1-bit at a time, via while loop, and also way
        // faster (~2x) than an unrolled loop that checks at 32, 16, 4,
        // 2, and 1 bit.
        //
        // Using a modulus of pow2 (which will get optimized to a bitwise
        // and with 0x3F or faster) is slightly slower than an if/then,
        // however, removing the if/then will likely optimize more branched
        // code as it removes conditional logic.

        // Calculate the number of leading zeros, and then zero-out
        // any overflowing bits, to avoid shl overflow when self.mant == 0.
        let shift = if self.mant == 0 { 0 } else { self.mant.leading_zeros() };
        shl(self, shift as i32);
        shift
    }

    // ROUND

    /// Lossy round float-point number to native mantissa boundaries.
    pub(crate) fn round_to_native<F>(&mut self)
        where F: Float
    {
        round_to_native::<F>(self)
    }

    // INTO

    /// Convert into lower-precision native float.
    pub fn into_float<F: Float>(mut self) -> F {
        self.round_to_native::<F>();
        into_float(self)
    }
}
