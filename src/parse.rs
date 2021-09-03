//! Parse byte iterators to float.

#![doc(hidden)]

use crate::algorithm::*;
use crate::digit::*;
use crate::exponent::*;
use crate::num::*;

// PARSERS
// -------

/// Parse the significant digits of the float.
///
/// * `integer`     - Slice containing the integer digits.
/// * `fraction`    - Slice containing the fraction digits.
fn parse_mantissa<'a, Iter1, Iter2>(mut integer: Iter1, mut fraction: Iter2) -> (u64, usize)
where
    Iter1: Iterator<Item = &'a u8>,
    Iter2: Iterator<Item = &'a u8>,
{
    let mut value: u64 = 0;
    // On overflow, validate that all the remaining characters are valid
    // digits, if not, return the first invalid digit. Otherwise,
    // calculate the number of truncated digits.
    while let Some(c) = integer.next() {
        value = match add_digit(value, to_digit(*c).unwrap()) {
            Some(v) => v,
            None => return (value, 1 + integer.count() + fraction.count()),
        };
    }
    while let Some(c) = fraction.next() {
        value = match add_digit(value, to_digit(*c).unwrap()) {
            Some(v) => v,
            None => return (value, 1 + fraction.count()),
        };
    }
    (value, 0)
}

/// Parse float from extracted float components.
///
/// * `integer`     - Cloneable, forward iterator over integer digits.
/// * `fraction`    - Cloneable, forward iterator over integer digits.
/// * `exponent`    - Parsed, 32-bit exponent.
///
/// # Preconditions
/// 1. The integer should not have leading zeros.
/// 2. The fraction should not have trailing zeros.
///
/// We cannot efficiently remove trailing zeros while only accepting a
/// forward iterator.
pub fn parse_float<'a, F, Iter1, Iter2>(integer: Iter1, fraction: Iter2, exponent: i32) -> F
where
    F: Float,
    Iter1: Iterator<Item = &'a u8> + Clone,
    Iter2: Iterator<Item = &'a u8> + Clone,
{
    // Parse the mantissa and attempt the fast and moderate-path algorithms.
    let (mantissa, truncated) = parse_mantissa(integer.clone(), fraction.clone());
    let is_truncated = truncated != 0;

    // Process the state to a float.
    if mantissa == 0 {
        // Literal 0, return early.
        // Value cannot be truncated, since truncation only occurs on
        // overflow or underflow.
        F::ZERO
    } else if !is_truncated {
        // Try the fast path, no mantissa truncation.
        let mant_exp = mantissa_exponent(exponent, fraction.clone().count(), 0);
        if let Some(float) = fast_path::<F>(mantissa, mant_exp) {
            float
        } else {
            fallback_path::<F, _, _>(integer, fraction, mantissa, exponent, mant_exp, is_truncated)
        }
    } else {
        let mant_exp = mantissa_exponent(exponent, fraction.clone().count(), truncated);
        fallback_path::<F, _, _>(integer, fraction, mantissa, exponent, mant_exp, is_truncated)
    }
}
