use super::cached::*;
use super::float::ExtendedFloat;
use super::num::*;

const POW10: [u64; 20] = [1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000, 1000000000, 10000000000, 100000000000, 1000000000000, 10000000000000, 100000000000000, 1000000000000000, 10000000000000000, 100000000000000000, 1000000000000000000, 10000000000000000000];

// FAST
// ----

/// Convert mantissa to exact value for a non-base2 power.
///
/// Returns the resulting float and if the value can be represented exactly.
fn fast_path<F>(mantissa: u64, exponent: i32)
    -> Option<F>
    where F: Float
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
        // Use powi, since it's correct, and faster on
        // the fast-path.
        let float = F::as_cast(mantissa);
        let base = F::as_cast(10i32);
        Some(float.mul(base.powi(exponent)))
    } else if exponent >= 0 && exponent <= max_exp + shift_exp {
        // Check to see if we have a disguised fast-path, where the
        // number of digits in the mantissa is very small, but and
        // so digits can be shifted from the exponent to the mantissa.
        // https://www.exploringbinary.com/fast-path-decimal-to-floating-point-conversion/
        let small_powers = POW10;
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
            let base = F::as_cast(10i32);
            Some(float.mul(base.powi(max_exp)))
        }
    } else {
        // Cannot be exactly represented, exponent too small or too big,
        // would require truncation.
        None
    }
}

// MODERATE
// --------

/// Multiply the floating-point by the exponent.
///
/// Multiply by pre-calculated powers of the base, modify the extended-
/// float, and return if new value and if the value can be represented
/// accurately.
fn multiply_exponent_extended<F>(fp: &mut ExtendedFloat, exponent: i32)
    where F: Float
{
    let powers = ExtendedFloat::get_powers();
    let exponent = exponent.saturating_add(powers.bias);
    let small_index = exponent % powers.step;
    let large_index = exponent / powers.step;
    if exponent < 0 {
        // Guaranteed underflow (assign 0).
        fp.mant = 0;
    } else if large_index as usize >= powers.large.len() {
        // Overflow (assign infinity)
        fp.mant = 1 << 63;
        fp.exp = 0x7FF;
    } else {
        // Within the valid exponent range, multiply by the large and small
        // exponents and return the resulting value.

        // Multiply by the small power.
        // Check if we can directly multiply by an integer, if not,
        // use extended-precision multiplication.
        match fp.mant.overflowing_mul(powers.get_small_int(small_index.as_usize())) {
            // Overflow, multiplication unsuccessful, go slow path.
            (_, true)     => {
                fp.normalize();
                fp.imul(&powers.get_small(small_index.as_usize()));
            },
            // No overflow, multiplication successful.
            (mant, false) => {
                fp.mant = mant;
                fp.normalize();
            },
        }

        // Multiply by the large power
        fp.imul(&powers.get_large(large_index.as_usize()));

        // Normalize the floating point (and the errors).
        fp.normalize();
    }
}

/// Create a precise native float using an intermediate extended-precision float.
///
/// Return the float approximation and if the value can be accurately
/// represented with mantissa bits of precision.
pub(super) fn moderate_path<F>(mantissa: u64, exponent: i32) -> F
    where F: Float
{
    let mut fp = ExtendedFloat { mant: mantissa, exp: 0 };
    multiply_exponent_extended::<F>(&mut fp, exponent);
    fp.into_float::<F>()
}

// PARSE
// -----

/// Parse non-power-of-two radix string to native float.
///
/// * `mantissa`        - Significant digits for float.
/// * `exponent`        - Mantissa exponent in decimal.
///
/// # Warning
/// The exponent is not the parsed exponent, for example:
///     "2.543" would have a mantissa of `2543` and an exponent of `-3`,
///     signifying it should be 2543 * 10^-3.
pub fn parse_float<F>(mantissa: u64, exponent: i32, truncated: bool) -> F
    where F: Float
{
    // Process the state to a float.
    if mantissa == 0 {
        // Literal 0, return early.
        // Value cannot be truncated, since truncation only occurs on
        // overflow or underflow.
        F::ZERO
    } else if !truncated {
        // Try the fast path, no mantissa truncation.
        if let Some(float) = fast_path::<F>(mantissa, exponent) {
            float
        } else {
            moderate_path(mantissa, exponent)
        }
    } else {
        moderate_path(mantissa, exponent)
    }
}
