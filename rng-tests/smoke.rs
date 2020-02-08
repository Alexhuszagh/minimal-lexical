//! Smoke test build from dtolnay.
//! All rights are his, merely used to test minimal-lexical.
//! https://github.com/serde-rs/json/issues/536#issuecomment-583708730

extern crate minimal_lexical;
extern crate rand_core;
extern crate rand_xorshift;
extern crate ryu;

use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

// HELPERS
// -------

// These functions are simple, resuable componetns

/// Find and parse sign and get remaining bytes.
#[inline]
fn parse_sign<'a>(bytes: &'a [u8]) -> (bool, &'a [u8]) {
    match bytes.get(0) {
        Some(&b'+') => (true, &bytes[1..]),
        Some(&b'-') => (false, &bytes[1..]),
        _           => (true, bytes)
    }
}

// Convert u8 to digit.
#[inline]
fn to_digit(c: u8) -> Option<u32> {
    (c as char).to_digit(10)
}

// Add digit to mantissa.
#[inline]
fn add_digit_u64(value: u64, digit: u32) -> Option<u64> {
    return value
        .checked_mul(10)?
        .checked_add(digit as u64)
}

// Add digit from exponent.
#[inline]
fn add_digit_i32(value: i32, digit: u32) -> Option<i32> {
    return value
        .checked_mul(10)?
        .checked_add(digit as i32)
}

// Subtract digit from exponent.
#[inline]
fn sub_digit_i32(value: i32, digit: u32) -> Option<i32> {
    return value
        .checked_mul(10)?
        .checked_sub(digit as i32)
}

// Convert character to digit.
#[inline]
fn is_digit(c: u8) -> bool {
    to_digit(c).is_some()
}

// Split buffer at index.
#[inline]
fn split_at_index<'a>(digits: &'a [u8], index: usize)
    -> (&'a [u8], &'a [u8])
{
    (&digits[..index], &digits[index..])
}

/// Consume until a an invalid digit is found.
///
/// - `digits`      - Slice containing 0 or more digits.
#[inline]
fn consume_digits<'a>(digits: &'a [u8])
    -> (&'a [u8], &'a [u8])
{
    // Consume all digits.
    let mut index = 0;
    while index < digits.len() && is_digit(digits[index]) {
        index += 1;
    }
    split_at_index(digits, index)
}

/// Convert usize into i32 without overflow.
///
/// This is needed to ensure when adjusting the exponent relative to
/// the mantissa we do not overflow for comically-long exponents.
#[inline]
fn into_i32(value: usize) -> i32 {
    if value > i32::max_value() as usize {
        i32::max_value()
    } else {
        value as i32
    }
}

// PARSERS
// -------

/// Parse the significant digits of the float.
///
/// * `integer`     - Slice containing the integer digits.
/// * `fraction`    - Slice containing the fraction digits.
fn parse_mantissa(integer: &[u8], fraction: &[u8])
    -> (u64, usize)
{
    let mut value: u64 = 0;
    let mut iter = integer.iter().chain(fraction.iter());
    // On overflow, validate that all the remaining characters are valid
    // digits, if not, return the first invalid digit. Otherwise,
    // calculate the number of truncated digits.
    while let Some(c) = iter.next() {
        value = match add_digit_u64(value, to_digit(*c).unwrap()) {
            Some(v) => v,
            None    => return (value, 1 + iter.count()),
        };
    }
    (value, 0)
}

/// Parse the exponent of the float.
///
/// * `exponent`    - Slice containing the exponent digits.
/// * `is_positive` - If the exponent sign is positive.
fn parse_exponent(exponent: &[u8], is_positive: bool) -> i32 {
    // Parse the sign bit or current data.
    let mut value: i32 = 0;
    match is_positive {
        true  => {
            for c in exponent {
                value = match add_digit_i32(value, to_digit(*c).unwrap()) {
                    Some(v) => v,
                    None    => return i32::max_value(),
                };
            }
        },
        false => {
            for c in exponent {
                value = match sub_digit_i32(value, to_digit(*c).unwrap()) {
                    Some(v) => v,
                    None    => return i32::min_value(),
                };
            }
        }
    }

    value
}

/// Parse float from input bytes, returning the float and the remaining bytes.
///
/// * `bytes`    - Array of bytes leading with float-data.
fn parse_float<'a, F>(bytes: &'a [u8])
    -> (F, &'a [u8])
    where F: minimal_lexical::Float
{
    // Parse the sign.
    let (is_positive, bytes) = parse_sign(bytes);

    // Extract and parse the float components:
    //  1. Integer
    //  2. Fraction
    //  3. Exponent
    let (integer_slc, bytes) = consume_digits(bytes);
    let (fraction_slc, bytes) = match bytes.first() {
        Some(&b'.') => consume_digits(&bytes[1..]),
        _           => (&bytes[..0], bytes),
    };
    let (mantissa, truncated) = parse_mantissa(integer_slc, fraction_slc);
    let (exponent, bytes) = match bytes.first() {
        Some(&b'e') | Some(&b'E') => {
            // Extract and parse the exponent.
            let (is_positive, bytes) = parse_sign(&bytes[1..]);
            let (exponent, bytes) = consume_digits(bytes);
            (parse_exponent(exponent, is_positive), bytes)
        },
        _                         =>  (0, bytes),
    };

    // Note: You may want to check and validate the float data here:
    //  1). Many floats require integer or fraction digits, if a fraction
    //      is present.
    //  2). All floats require either integer or fraction digits.
    //  3). Some floats do not allow a '+' sign before the significant digits.
    //  4). Many floats require exponent digits after the exponent symbol.
    //  5). Some floats do not allow a '+' sign before the exponent.

    // Calculate the exponent relative to the significant digits.
    let mantissa_exponent;
    let fraction_digits = fraction_slc.len();
    if fraction_digits > truncated {
        mantissa_exponent = exponent.saturating_sub(into_i32(fraction_digits - truncated));
    } else {
        mantissa_exponent = exponent.saturating_add(into_i32(truncated - fraction_digits));
    }

    // Create the float and return our data.
    let is_truncated = truncated != 0;
    let mut float: F = minimal_lexical::create_float(mantissa, mantissa_exponent, is_truncated);
    if !is_positive {
        float = -float;
    }

    (float, bytes)
}

pub fn main() {
    let mut rng = XorShiftRng::from_seed([0; 16]);
    let mut buffer = ryu::Buffer::new();
    loop {
        let input = f64::from_bits(rng.next_u64());
        if input.is_finite() {
            let printed = buffer.format_finite(input);
            let (output, rest) = parse_float::<f64>(printed.as_bytes());
            assert_eq!(output, input);
            assert_eq!(rest, b"");
        }
    }
}
