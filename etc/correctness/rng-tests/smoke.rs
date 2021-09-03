//! Smoke test build from dtolnay.
//! All rights are his, merely used to test minimal-lexical.
//! https://github.com/serde-rs/json/issues/536#issuecomment-583708730

extern crate rand_core;
extern crate rand_xorshift;
extern crate ryu;

use rand_core::{RngCore, SeedableRng};
use rand_xorshift::XorShiftRng;

mod _common;
use self::_common::parse_float;

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
