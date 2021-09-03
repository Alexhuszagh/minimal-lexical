//! Exhaustively test every f32 value.

mod _common;

use self::_common::parse_float;
use minimal_lexical::Float;

pub fn main() {
    let mut buffer = ryu::Buffer::new();
    for i in 0..=f32::EXPONENT_MASK {
        let input = f32::from_bits(i);
        if i % 100000 == 0 {
            println!("Processed {} records.", i);
        }
        let printed = buffer.format_finite(input);
        let (output, rest) = parse_float::<f32>(printed.as_bytes());
        assert_eq!(output, input);
        assert_eq!(rest, b"");
    }
}
