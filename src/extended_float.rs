//! Estimate the error in an 80-bit approximation of a float.
//!
//! This estimates the error in a floating-point representation.
//!
//! This implementation is loosely based off the Golang implementation,
//! found here:
//!     https://golang.org/src/strconv/atof.go

use crate::float::*;
use crate::num::*;
use crate::powers::*;
use crate::rounding::*;

// ERRORS
// ------

/// Check if the error is accurate with a round-nearest rounding scheme.
#[inline]
fn nearest_error_is_accurate(errors: u64, fp: &ExtendedFloat, extrabits: u64) -> bool {
    // Round-to-nearest, need to use the halfway point.
    if extrabits == 65 {
        // Underflow, we have a shift larger than the mantissa.
        // Representation is valid **only** if the value is close enough
        // overflow to the next bit within errors. If it overflows,
        // the representation is **not** valid.
        !fp.mant.overflowing_add(errors).1
    } else {
        let mask: u64 = lower_n_mask(extrabits);
        let extra: u64 = fp.mant & mask;

        // Round-to-nearest, need to check if we're close to halfway.
        // IE, b10100 | 100000, where `|` signifies the truncation point.
        let halfway: u64 = lower_n_halfway(extrabits);
        let cmp1 = halfway.wrapping_sub(errors) < extra;
        let cmp2 = extra < halfway.wrapping_add(errors);

        // If both comparisons are true, we have significant rounding error,
        // and the value cannot be exactly represented. Otherwise, the
        // representation is valid.
        !(cmp1 && cmp2)
    }
}

#[inline(always)]
fn error_scale() -> u32 {
    8
}

#[inline(always)]
fn error_halfscale() -> u32 {
    error_scale() / 2
}

#[inline]
fn error_is_accurate<F: Float>(count: u32, fp: &ExtendedFloat) -> bool {
    // Determine if extended-precision float is a good approximation.
    // If the error has affected too many units, the float will be
    // inaccurate, or if the representation is too close to halfway
    // that any operations could affect this halfway representation.
    // See the documentation for dtoa for more information.
    let bias = -(F::EXPONENT_BIAS - F::MANTISSA_SIZE);
    let denormal_exp = bias - 63;
    // This is always a valid u32, since (denormal_exp - fp.exp)
    // will always be positive and the significand size is {23, 52}.
    let extrabits = match fp.exp <= denormal_exp {
        true => 64 - F::MANTISSA_SIZE + denormal_exp - fp.exp,
        false => 63 - F::MANTISSA_SIZE,
    };

    // Our logic is as follows: we want to determine if the actual
    // mantissa and the errors during calculation differ significantly
    // from the rounding point. The rounding point for round-nearest
    // is the halfway point, IE, this when the truncated bits start
    // with b1000..., while the rounding point for the round-toward
    // is when the truncated bits are equal to 0.
    // To do so, we can check whether the rounding point +/- the error
    // are >/< the actual lower n bits.
    //
    // For whether we need to use signed or unsigned types for this
    // analysis, see this example, using u8 rather than u64 to simplify
    // things.
    //
    // # Comparisons
    //      cmp1 = (halfway - errors) < extra
    //      cmp1 = extra < (halfway + errors)
    //
    // # Large Extrabits, Low Errors
    //
    //      extrabits = 8
    //      halfway          =  0b10000000
    //      extra            =  0b10000010
    //      errors           =  0b00000100
    //      halfway - errors =  0b01111100
    //      halfway + errors =  0b10000100
    //
    //      Unsigned:
    //          halfway - errors = 124
    //          halfway + errors = 132
    //          extra            = 130
    //          cmp1             = true
    //          cmp2             = true
    //      Signed:
    //          halfway - errors = 124
    //          halfway + errors = -124
    //          extra            = -126
    //          cmp1             = false
    //          cmp2             = true
    //
    // # Conclusion
    //
    // Since errors will always be small, and since we want to detect
    // if the representation is accurate, we need to use an **unsigned**
    // type for comparisons.

    let extrabits = extrabits as u64;
    let errors = count as u64;
    if extrabits > 65 {
        // Underflow, we have a literal 0.
        return true;
    }

    nearest_error_is_accurate(errors, fp, extrabits)
}

// MODERATE PATH
// -------------

/// Multiply the floating-point by the exponent.
///
/// Multiply by pre-calculated powers of the base, modify the extended-
/// float, and return if new value and if the value can be represented
/// accurately.
fn multiply_exponent_extended<F>(fp: &mut ExtendedFloat, exponent: i32, truncated: bool) -> bool
where
    F: Float,
{
    if exponent < MIN_DENORMAL_EXP10 {
        // Guaranteed underflow (assign 0).
        fp.mant = 0;
        true
    } else if exponent > MAX_NORMAL_EXP10 {
        // Overflow (assign infinity)
        fp.mant = 1 << 63;
        fp.exp = 0x7FF;
        true
    } else {
        // Within the valid exponent range, multiply by the large and small
        // exponents and return the resulting value.

        // Track errors to as a factor of unit in last-precision.
        let mut errors: u32 = 0;
        if truncated {
            errors += error_halfscale();
        }

        // Infer the binary exponent from the power of 10.
        // Adjust this exponent to the fact the value is normalized (1<<63).
        let exp = -63 + (217706 * exponent as i64 >> 16);
        let mant = POWERS_OF_10[(exponent - MIN_DENORMAL_EXP10) as usize].0;
        let large = ExtendedFloat {
            mant,
            exp: exp as i32,
        };

        // Normalize fp and multiple by large.
        fp.normalize();
        fp.imul(&large);
        if errors > 0 {
            errors += 1;
        }
        errors += error_halfscale();

        // Normalize the floating point (and the errors).
        let shift = fp.normalize();
        errors <<= shift;

        error_is_accurate::<F>(errors, &fp)
    }
}

/// Create a precise native float using an intermediate extended-precision float.
///
/// Return the float approximation and if the value can be accurately
/// represented with mantissa bits of precision.
#[inline]
pub(super) fn moderate_path<F>(mantissa: u64, exponent: i32, truncated: bool) -> (F, bool)
where
    F: Float,
{
    let mut fp = ExtendedFloat {
        mant: mantissa,
        exp: 0,
    };
    let valid = multiply_exponent_extended::<F>(&mut fp, exponent, truncated);
    if valid {
        let float = fp.into_float::<F>();
        (float, true)
    } else {
        // Need the slow-path algorithm.
        let float = fp.into_downward_float::<F>();
        (float, false)
    }
}

// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moderate_path_test() {
        let (f, valid) = moderate_path::<f64>(1234567890, -1, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.0);

        let (f, valid) = moderate_path::<f64>(1234567891, -1, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.1);

        let (f, valid) = moderate_path::<f64>(12345678912, -2, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.12);

        let (f, valid) = moderate_path::<f64>(123456789123, -3, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.123);

        let (f, valid) = moderate_path::<f64>(1234567891234, -4, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.1234);

        let (f, valid) = moderate_path::<f64>(12345678912345, -5, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.12345);

        let (f, valid) = moderate_path::<f64>(123456789123456, -6, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.123456);

        let (f, valid) = moderate_path::<f64>(1234567891234567, -7, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.1234567);

        let (f, valid) = moderate_path::<f64>(12345678912345679, -8, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 123456789.12345679);

        let (f, valid) = moderate_path::<f64>(4628372940652459, -17, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 0.04628372940652459);

        let (f, valid) = moderate_path::<f64>(26383446160308229, -272, false);
        assert!(valid, "should be valid");
        assert_eq!(f, 2.6383446160308229e-256);

        let (_, valid) = moderate_path::<f64>(26383446160308230, -272, false);
        assert!(!valid, "should be invalid");
    }
}
