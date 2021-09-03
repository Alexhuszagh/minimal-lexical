//! Fast estimation of the accurate representation of a float.
//!
//! Based off the Golang implementation of the Eisel-Lemire algorithm,
//! found here:
//!     https://github.com/golang/go/blob/2ebe77a2fda1ee9ff6fd9a3e08933ad1ebaea039/src/strconv/eisel_lemire.go
//!
//! Which, itself was based off of the Wuff's implementation:
//!     https://github.com/google/wuffs/blob/ba3818cb6b473a2ed0b38ecfc07dbbd3a97e8ae7/internal/cgen/base/floatconv-submodule-code.c
//!
//! The original algorithm may be found here:
//!     https://github.com/lemire/fast_double_parser
//!
//! And an in-depth blogpost describing the algorithms may be found here:
//!     https://nigeltao.github.io/blog/2020/eisel-lemire.html
//!
//! # Magic Number Generation
//!
//! ```python
//! import math
//!
//! def get_range(max_exp, bitshift):
//!     den = 1 << bitshift
//!     num = int(math.ceil(math.log2(10) * den))
//!     for exp10 in range(0, max_exp):
//!         exp2_exact = int(math.log2(10**exp10))
//!         exp2_guess = num * exp10 // den
//!         if exp2_exact != exp2_guess:
//!             raise ValueError(f'{exp10}')
//!     return num, den
//! ```
//!
//! For 64-bit and smaller floats, we therefore need a bitshift of 16,
//! so our magic number is `217706`. For 128-bit floats, we need a bitshift
//! of >= 25, so we round up to 32, and therefore need a magic number
//! of `14267572528`. Note that due to the storage requirements,
//! 128-bit floats do not currently use this algorithm.

#![doc(hidden)]

use crate::extended_float;
use crate::num::*;
use crate::powers::*;

// MUL
// ---

/// Multiply two unsigned, integral values, and return the hi and lo product.
#[inline(always)]
pub fn mul(x: u64, y: u64) -> (u64, u64) {
    // Extract high-and-low masks.
    let x1 = x >> u64::HALF;
    let x0 = x & u64::LOMASK;
    let y1 = y >> u64::HALF;
    let y0 = y & u64::LOMASK;

    // Get our products
    let w0 = x0 * y0;
    let tmp = (x1 * y0) + (w0 >> u64::HALF);
    let w1 = tmp & u64::LOMASK;
    let w2 = tmp >> u64::HALF;
    let w1 = w1 + x0 * y1;
    let hi = (x1 * y1) + w2 + (w1 >> u64::HALF);
    let lo = x.wrapping_mul(y);

    (hi, lo)
}

// SHIFT
// -----

/// Shift significant digits to at most the carry bit
/// The carry bit is 1 above the hidden bit, in the exponent,
/// or mantissa size + 2.
#[inline(always)]
fn shift_to_carry(x_hi: u64, exp2: i32, carry_shift: i32) -> (u64, i32) {
    // Carry out the shift
    let msb_shift = u64::FULL - 1;
    let msb = x_hi >> msb_shift;
    let shift = msb.as_i32() + carry_shift;
    let mantissa = x_hi >> shift;
    let exp2 = exp2 - (1i32 ^ msb.as_i32());

    (mantissa, exp2)
}

// TO FLOAT
// --------

/// Convert mantissa and binary exponent to floating-point representation.
///
/// This function expects the following things:
///     1). The highest mantissa bit set is 1 above the carry bit.
///     2). The lowest mantissa bit set is the carry bit.
///         That is, 2 above the hidden bit, or 1 above the hidden bit.
///     3). The binary exponent is adjusted for the exponent bias.
#[inline(always)]
fn to_float<F>(mantissa: u64, exp: i32) -> (F, bool)
where
    F: Float,
{
    // Check denormal values for underflow.
    if exp <= -(F::MANTISSA_SIZE + 2) {
        // Have a literal zero. If we shift the bits, we'll get 0.
        return (F::ZERO, true);
    } else if exp <= 0 {
        // We don't actually care about the accuracy here,
        // since we're going straight to the extended-float algorithm.
        return (F::ZERO, false);
    }

    // Get our raw bits.
    let mut exp = F::Unsigned::as_cast(exp);
    let mut mantissa = F::Unsigned::as_cast(mantissa);

    // Round-nearest, tie-even.
    let zero = F::Unsigned::ZERO;
    let one = F::Unsigned::as_cast(1);
    mantissa += mantissa & one;

    // Shift them into position.
    mantissa >>= 1i32;
    let precision = F::MANTISSA_SIZE + 1;
    if mantissa >> precision > zero {
        mantissa >>= 1i32;
        exp += one;
    }

    // Check our mantissa representation is valid, that is,
    // we didn't have a bit mantissa or hidden bit set.
    let mask = F::MANTISSA_MASK | F::HIDDEN_BIT_MASK;
    debug_assert!(mantissa & mask == mantissa);

    // Check for overflow, if so, return a literal infinity.
    let max_exp = F::MAX_EXPONENT + F::EXPONENT_BIAS;
    if exp >= F::Unsigned::as_cast(max_exp) {
        let float = F::from_bits(F::INFINITY_BITS);
        return (float, true);
    }

    // Should fail, we shouldn't have any exponent bits set.
    mantissa &= F::MANTISSA_MASK;
    exp <<= F::MANTISSA_SIZE;
    let bits = exp | mantissa;

    (F::from_bits(bits), true)
}

// EISEL-LEMIRE
// ------------

/// Create a precise native float using the Eisel-Lemire algorithm.
///
/// NOTE: If the Eisel-Lamire algorithm cannot differentiate a halfway
/// representation, it cannot determine whether to round up or down
/// to determine the correct `b` value for big-float determination.
///
/// In that case, we fall back to extended-float to determine that
/// representation.
#[inline]
pub fn eisel_lemire<F>(mantissa: u64, exponent: i32) -> (F, bool)
where
    F: Float,
{
    // Check if the value is outside of our max range:
    //  If the value is above our max range, we have to have an infinity,
    //  and we have an exact representation (1e348) is infinity, which
    //  is the minimum possible value above this range.
    //
    // For negative values, we're guaranteed to have 0 as well:
    //  with 2470328229206232720e-342, we round to 0, while with
    //  2470328229206232721e-342, we round up to 5e-324. Both of these
    //  contain the maximum number of mantissa digits (19), so our
    //  base-10 exponent cannot get any lower.
    //
    // Note that this only checks beyond the limits of f64, we do
    // checks for narrower types further in.
    if exponent < MIN_DENORMAL_EXP10 {
        return (F::ZERO, true);
    } else if exponent > MAX_NORMAL_EXP10 {
        let float = F::from_bits(F::INFINITY_BITS);
        return (float, true);
    }

    // Normalize the mantissa, and calculate the bias.
    let ctlz = mantissa.leading_zeros() as i32;
    let mantissa = mantissa << ctlz;
    let bias = F::EXPONENT_BIAS - F::MANTISSA_SIZE;

    // Need to convert our base 10 exponent to base 2, as an estimate.
    // See module documentation for how we generated these magic numbers.
    let unbiased_exp2 = (217706 * exponent as i64) >> 16;
    let exp2 = unbiased_exp2 as i32 + (u64::FULL + bias) - ctlz;

    // Now need to get our extended, power of 10:
    let (exp10_hi, exp10_lo) = POWERS_OF_10[(exponent - MIN_DENORMAL_EXP10) as usize];
    let exp10_hi = exp10_hi;
    let exp10_lo = exp10_lo;
    let (mut x_hi, mut x_lo) = mul(mantissa, exp10_hi);

    // NOTE:
    //  From here we make a few differences from the Lemire implementation,
    //  to streamline integration with the slow path algorithm.
    //
    //  192-BIT
    //  -------
    //
    //  When we check for halfway representations, for the extended
    //  192-bit representation, we assume the following logic:
    //  - If we have `x_hi & mask == mask` and wrapping behavior,
    //      then we are close to a halfway representation, but 1-bit below.
    //  - If `merged_hi & mask == mask` and `merged_lo + 1 == 0`, then
    //      we are within 1-bit of the halfway representation.
    //  In this case, we should add 1-bit, to get to the halfway
    //  representation, and round-down, so we can get our `b` representation
    //  to differentiate `b` from `b+u` near to `b+h`.
    //
    //  AFTER-SHIFTING
    //  --------------
    //
    //  After shifting and checking for truncated bits, we have shifted
    //  to the carry bit + 1. This means we are 2 bits above the hidden
    //  bit, so we have a halfway representation if `mantissa & 3 == 1`,
    //  and the truncated bits were 0 (`x_lo == 0` and `x_hi & mask == 0`).
    //  Here, since we're at least at a halfway representation, round-down
    //  so we get `b`. We're already **at least** at a halfway representation,
    //  so we should not add any bits to the shifted mantissa.

    // Now need to check for a wider approximation.
    let carry_size = F::MANTISSA_SIZE + 2;
    let carry_shift = u64::FULL - carry_size - 1;
    let mask = (1u64 << carry_shift) - 1;
    if x_hi & mask == mask && x_lo.wrapping_add(mantissa) < mantissa {
        let (y_hi, y_lo) = mul(mantissa, exp10_lo);
        let mut merged_hi = x_hi;
        let merged_lo = x_lo.wrapping_add(y_hi);
        if merged_lo < x_lo {
            merged_hi += 1;
        }

        // Check for a halfway representation.
        if merged_hi & mask == mask
            && merged_lo.wrapping_add(1) == 0
            && y_lo.wrapping_add(mantissa) < mantissa
        {
            // We don't actually care about the accuracy here,
            // since we're going straight to the extended-float algorithm.
            return (F::ZERO, false);
        } else {
            x_hi = merged_hi;
            x_lo = merged_lo;
        }
    }

    // Shift to the carry bit (IE, mantissa size + 2).
    let (mantissa, exp2) = shift_to_carry(x_hi, exp2, carry_shift);

    // Check for a halfway representation.
    if x_lo == 0 && x_hi & mask == 0 && mantissa & 3 == 1 {
        // We don't actually care about the accuracy here,
        // since we're going straight to the extended-float algorithm.
        return (F::ZERO, false);
    }

    to_float(mantissa, exp2)
}

/// Create a precise native float using the Eisel-Lemire algorithm.
///
/// Note that the Eisel-Lemire algorithm may not be accurate if
/// truncated digits occur, so we do a second pass with the
/// mantissa + 1 (to solve any halfway issues with truncated
/// digits), and if the two values are the same, return true.
/// This avoids any costly error estimation, since if `mantissa`
/// `mantissa+1` are the same, we cannot have had a halfway case.
///
/// Note that if we cannot determine a valid representation,
/// we fall back to the extended-float moderate path, so we can
/// get an accurate, base representation for big-integer
/// algorithms.
#[inline]
pub fn moderate_path<F>(mantissa: u64, exponent: i32, truncated: bool) -> (F, bool)
where
    F: Float,
{
    let (float, valid) = eisel_lemire(mantissa, exponent);
    if valid {
        if !truncated {
            (float, true)
        } else {
            let mantissa_up = mantissa + 1;
            let (float_up, valid) = eisel_lemire(mantissa_up, exponent);
            if valid && float == float_up {
                (float, true)
            } else {
                (float, false)
            }
        }
    } else {
        // If the first representation failed, try the extended-float
        // algorithm, since it's a lot faster for small, denormal floats.
        extended_float::moderate_path::<F>(mantissa, exponent, truncated)
    }
}
