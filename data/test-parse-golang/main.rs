// Copyright 2021, Alex Huszagh. Unlicensed.
// See https://unlicense.org/

extern crate minimal_lexical;

use std::io::prelude::*;
use std::path::PathBuf;
use std::{env, fs, io};

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

/// Return the `data` directory path.
pub fn data_dir() -> PathBuf {
    let mut dir = project_dir();
    dir.push("data");
    dir.push("test-parse-golang");
    dir.push("parse-number-fxx-test-data");
    dir.push("data");
    dir
}

fn run_test(line: &str) {
    // Tests have the following format:
    //      hhhh ssssssss dddddddddddddddddd ....
    // The `hhhh` part is the hexadecimal representation for f16,
    // the `ssssssss` part is the hexadecimal representation of f32,
    // the `dddddddddddddddddd` is the hex representation of f64,
    // and the remaining bytes are the string to parse.
    let hex32 = line[5..13].to_lowercase();
    let hex64 = line[14..30].to_lowercase();
    let string = &line[31..];

    let float32: f32 = parse_float(string.as_bytes()).0;
    let float64: f64 = parse_float(string.as_bytes()).0;
    assert_eq!(hex32, format!("{:0>8x}", float32.to_bits()));
    assert_eq!(hex64, format!("{:0>16x}", float64.to_bits()));
}

fn main() {
    // Iterate over all .txt files in the directory.
    let paths = fs::read_dir(data_dir()).expect("Please update the Git submodule");
    for direntry in paths {
        let path = direntry.unwrap().path();
        if path.extension().unwrap() == "txt" {
            // Have a data file, parse and run the tests.
            let filename = path.file_name().unwrap().to_str().unwrap();
            println!("Running Test: {}", filename);
            let file = fs::File::open(path).unwrap();
            let reader = io::BufReader::new(file);
            let mut count: usize = 0;
            for line in reader.lines() {
                let line = line.unwrap();
                run_test(&line);
                count += 1;
            }
            println!("Ran {} tests.", count);
        }
    }
}