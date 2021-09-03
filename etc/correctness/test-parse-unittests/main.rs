// Copyright 2018, Alex Huszagh. Unlicensed.
// See https://unlicense.org/

#![allow(non_snake_case)]

extern crate minimal_lexical;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

// HELPERS
// -------

// These functions are simple, resuable componetns

/// Find and parse sign and get remaining bytes.
#[inline]
fn parse_sign<'a>(bytes: &'a [u8]) -> (bool, &'a [u8]) {
    match bytes.get(0) {
        Some(&b'+') => (true, &bytes[1..]),
        Some(&b'-') => (false, &bytes[1..]),
        _ => (true, bytes),
    }
}

// Convert u8 to digit.
#[inline]
fn to_digit(c: u8) -> Option<u32> {
    (c as char).to_digit(10)
}

// Add digit from exponent.
#[inline]
fn add_digit_i32(value: i32, digit: u32) -> Option<i32> {
    return value.checked_mul(10)?.checked_add(digit as i32);
}

// Subtract digit from exponent.
#[inline]
fn sub_digit_i32(value: i32, digit: u32) -> Option<i32> {
    return value.checked_mul(10)?.checked_sub(digit as i32);
}

// Convert character to digit.
#[inline]
fn is_digit(c: u8) -> bool {
    to_digit(c).is_some()
}

// Split buffer at index.
#[inline]
fn split_at_index<'a>(digits: &'a [u8], index: usize) -> (&'a [u8], &'a [u8]) {
    (&digits[..index], &digits[index..])
}

/// Consume until a an invalid digit is found.
///
/// - `digits`      - Slice containing 0 or more digits.
#[inline]
fn consume_digits<'a>(digits: &'a [u8]) -> (&'a [u8], &'a [u8]) {
    // Consume all digits.
    let mut index = 0;
    while index < digits.len() && is_digit(digits[index]) {
        index += 1;
    }
    split_at_index(digits, index)
}

// Trim leading 0s.
#[inline]
fn ltrim_zero<'a>(bytes: &'a [u8]) -> &'a [u8] {
    let count = bytes.iter().take_while(|&&si| si == b'0').count();
    &bytes[count..]
}

// Trim trailing 0s.
#[inline]
fn rtrim_zero<'a>(bytes: &'a [u8]) -> &'a [u8] {
    let count = bytes.iter().rev().take_while(|&&si| si == b'0').count();
    let index = bytes.len() - count;
    &bytes[..index]
}

// PARSERS
// -------

/// Parse the exponent of the float.
///
/// * `exponent`    - Slice containing the exponent digits.
/// * `is_positive` - If the exponent sign is positive.
fn parse_exponent(exponent: &[u8], is_positive: bool) -> i32 {
    // Parse the sign bit or current data.
    let mut value: i32 = 0;
    match is_positive {
        true => {
            for c in exponent {
                value = match add_digit_i32(value, to_digit(*c).unwrap()) {
                    Some(v) => v,
                    None => return i32::max_value(),
                };
            }
        },
        false => {
            for c in exponent {
                value = match sub_digit_i32(value, to_digit(*c).unwrap()) {
                    Some(v) => v,
                    None => return i32::min_value(),
                };
            }
        },
    }

    value
}

/// Parse float from input bytes, returning the float and the remaining bytes.
///
/// * `bytes`    - Array of bytes leading with float-data.
fn parse_float<'a, F>(bytes: &'a [u8]) -> (F, &'a [u8])
where
    F: minimal_lexical::Float,
{
    // Parse the sign.
    let (is_positive, bytes) = parse_sign(bytes);

    // Extract and parse the float components:
    let (integer_slc, bytes) = consume_digits(bytes);
    let (fraction_slc, bytes) = match bytes.first() {
        Some(&b'.') => consume_digits(&bytes[1..]),
        _ => (&bytes[..0], bytes),
    };
    let (exponent, bytes) = match bytes.first() {
        Some(&b'e') | Some(&b'E') => {
            // Extract and parse the exponent.
            let (is_positive, bytes) = parse_sign(&bytes[1..]);
            let (exponent, bytes) = consume_digits(bytes);
            (parse_exponent(exponent, is_positive), bytes)
        },
        _ => (0, bytes),
    };

    // Trim leading and trailing zeros.
    let integer_slc = ltrim_zero(integer_slc);
    let fraction_slc = rtrim_zero(fraction_slc);

    // Create the float and return our data.
    let mut float: F =
        minimal_lexical::parse_float(integer_slc.iter(), fraction_slc.iter(), exponent);
    if !is_positive {
        float = -float;
    }

    (float, bytes)
}

// STRUCTS
// Derived structs for the Toml parser.

#[derive(Debug, Deserialize)]
struct StrtodTests {
    negativeFormattingTests: Vec<String>,
    FormattingTests: Vec<FormattingTest>,
    ConversionTests: Vec<ConversionTest>,
}

#[derive(Debug, Deserialize)]
struct FormattingTest {
    UID: String,
    str: String,
    hex: String,
    int: String,
}

#[derive(Debug, Deserialize)]
struct ConversionTest {
    UID: String,
    str: String,
    hex: String,
    int: String,
}

// PATH

/// Return the `target/debug` or `target/release` directory path.
pub fn build_dir() -> PathBuf {
    env::current_exe()
        .expect("unittest executable path")
        .parent()
        .expect("debug/release directory")
        .to_path_buf()
}

/// Return the `target` directory path.
pub fn target_dir() -> PathBuf {
    build_dir().parent().expect("target directory").to_path_buf()
}

/// Return the project directory path.
pub fn project_dir() -> PathBuf {
    target_dir().parent().expect("project directory").to_path_buf()
}

fn run_test(string: &str, hex: &str) {
    // This parser doesn't handle literal NaNs/infs.
    let lower = string.to_lowercase();
    if !lower.contains("nan") && !lower.contains("inf") {
        let float: f64 = parse_float(string.as_bytes()).0;
        let int: u64 = float.to_bits();
        // Rust literals for NaN are not standard conforming:
        // Rust uses 0x7ff8000000000000, not 0x7ff0000000000001
        // We want to pad on the left with 0s, up to 16 characters.
        if float.is_finite() {
            assert_eq!(hex, format!("{:0>16x}", int));
        }
    }
}

fn run_tests(tests: StrtodTests) {
    let formatting_tests_count = tests.FormattingTests.len();
    let conversion_tests_count = tests.ConversionTests.len();
    for test in tests.FormattingTests {
        run_test(&test.str, &test.hex)
    }
    for test in tests.ConversionTests {
        run_test(&test.str, &test.hex)
    }
    println!("Ran {} formatting tests.", formatting_tests_count);
    println!("Ran {} conversion tests.", conversion_tests_count);
    println!("");
}

fn parse_tests(name: &str) -> StrtodTests {
    let mut test_path = project_dir();
    test_path.push("test-parse-unittests");
    test_path.push(name);
    let test_data = read_to_string(test_path).unwrap();

    toml::from_str(&test_data).unwrap()
}

fn main() {
    let filenames = ["strtod_tests.toml", "rust_parse_tests.toml"];
    for filename in filenames.iter() {
        println!("Running Test: {}", filename);
        run_tests(parse_tests(filename));
    }
}
