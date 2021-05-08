//! Algorithms to efficiently convert strings to floats.

use super::bhcomp::*;
use super::lemire::*;
use super::num::*;
use super::small_powers::*;

// FAST
// ----

/// Convert mantissa to exact value for a non-base2 power.
///
/// Returns the resulting float and if the value can be represented exactly.
pub(crate) fn fast_path<F>(mantissa: u64, exponent: i32) -> Option<F>
where
    F: Float,
{
    // `mantissa >> (F::MANTISSA_SIZE+1) != 0` effectively checks if the
    // value has a no bits above the hidden bit, which is what we want.
    let (min_exp, max_exp) = F::exponent_limit();
    let shift_exp = F::mantissa_limit();
    let mantissa_size = F::MANTISSA_SIZE + 1;
    if mantissa >> mantissa_size != 0 {
        // Would require truncation of the mantissa.
        None
    } else if exponent == 0 {
        // 0 exponent, same as value, exact representation.
        let float = F::as_cast(mantissa);
        Some(float)
    } else if exponent >= min_exp && exponent <= max_exp {
        // Value can be exactly represented, return the value.
        // Do not use powi, since powi can incrementally introduce
        // error.
        let float = F::as_cast(mantissa);
        Some(float.pow10(exponent))
    } else if exponent >= 0 && exponent <= max_exp + shift_exp {
        // Check to see if we have a disguised fast-path, where the
        // number of digits in the mantissa is very small, but and
        // so digits can be shifted from the exponent to the mantissa.
        // https://www.exploringbinary.com/fast-path-decimal-to-floating-point-conversion/
        let small_powers = POW10_64;
        let shift = exponent - max_exp;
        let power = small_powers[shift.as_usize()];

        // Compute the product of the power, if it overflows,
        // prematurely return early, otherwise, if we didn't overshoot,
        // we can get an exact value.
        let value = mantissa.checked_mul(power)?;
        if value >> mantissa_size != 0 {
            None
        } else {
            // Use powi, since it's correct, and faster on
            // the fast-path.
            let float = F::as_cast(value);
            Some(float.pow10(max_exp))
        }
    } else {
        // Cannot be exactly represented, exponent too small or too big,
        // would require truncation.
        None
    }
}

// FALLBACK
// --------

/// Fallback path when the fast path does not work.
///
/// Uses the moderate path, if applicable, otherwise, uses the slow path
/// as required.
pub(crate) fn fallback_path<'a, F, Iter1, Iter2>(
    integer: Iter1,
    fraction: Iter2,
    mantissa: u64,
    exponent: i32,
    mantissa_exponent: i32,
    truncated: bool,
) -> F
where
    F: Float,
    Iter1: Iterator<Item = &'a u8> + Clone,
    Iter2: Iterator<Item = &'a u8> + Clone,
{
    // Moderate path (use an extended 80-bit representation).
    let (float, valid) = moderate_path::<F>(mantissa, mantissa_exponent, truncated);
    if valid || float.is_special() {
        return float;
    }

    // Slow path, fast path didn't work.
    return bhcomp(float, integer, fraction, exponent);
}

// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn float_fast_path_test() {
        // valid
        let mantissa = (1 << f32::MANTISSA_SIZE) - 1;
        let (min_exp, max_exp) = f32::exponent_limit();
        for exp in min_exp..max_exp + 1 {
            let f = fast_path::<f32>(mantissa, exp);
            assert!(f.is_some(), "should be valid {:?}.", (mantissa, exp));
        }

        // Check slightly above valid exponents
        let f = fast_path::<f32>(123, 15);
        assert_eq!(f, Some(1.23e+17));

        // Exponent is 1 too high, pushes over the mantissa.
        let f = fast_path::<f32>(123, 16);
        assert!(f.is_none());

        // Mantissa is too large, checked_mul should overflow.
        let f = fast_path::<f32>(mantissa, 11);
        assert!(f.is_none());

        // invalid mantissa
        #[cfg(feature = "radix")]
        {
            let (_, max_exp) = f64::exponent_limit(3);
            let f = fast_path::<f32>(1 << f32::MANTISSA_SIZE, 3, max_exp + 1);
            assert!(f.is_none(), "invalid mantissa");
        }

        // invalid exponents
        let (min_exp, max_exp) = f32::exponent_limit();
        let f = fast_path::<f32>(mantissa, min_exp - 1);
        assert!(f.is_none(), "exponent under min_exp");

        let f = fast_path::<f32>(mantissa, max_exp + 1);
        assert!(f.is_none(), "exponent above max_exp");
    }

    #[test]
    fn double_fast_path_test() {
        // valid
        let mantissa = (1 << f64::MANTISSA_SIZE) - 1;
        let (min_exp, max_exp) = f64::exponent_limit();
        for exp in min_exp..max_exp + 1 {
            let f = fast_path::<f64>(mantissa, exp);
            assert!(f.is_some(), "should be valid {:?}.", (mantissa, exp));
        }

        // invalid mantissa
        #[cfg(feature = "radix")]
        {
            let (_, max_exp) = f64::exponent_limit(3);
            let f = fast_path::<f64>(1 << f64::MANTISSA_SIZE, 3, max_exp + 1);
            assert!(f.is_none(), "invalid mantissa");
        }

        // invalid exponents
        let (min_exp, max_exp) = f64::exponent_limit();
        let f = fast_path::<f64>(mantissa, min_exp - 1);
        assert!(f.is_none(), "exponent under min_exp");

        let f = fast_path::<f64>(mantissa, max_exp + 1);
        assert!(f.is_none(), "exponent above max_exp");

        assert_eq!(Some(0.04628372940652459), fast_path::<f64>(4628372940652459, -17));
        assert_eq!(None, fast_path::<f64>(26383446160308229, -272));
    }
}
