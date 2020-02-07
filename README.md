minimal_lexical
===============

This is a minimal version of [rust-lexical](https://github.com/Alexhuszagh/rust-lexical), meant to allow efficient round-trip float parsing. This does not implement a correct float parser, however, it will be accurate for round-trip values.

# Getting Started

```rust
extern crate minimal_lexical;

// Let's say we want to parse "1.2345".
// First, we have an external parser parse all the significant digits
// into a u64 (`12345`), and then we have the exponent adjusted to
// the mantissa (`-4`, since the value is `12345e-4`). Finally,
// we store whether digits were truncated from the mantissa during 
// parsing (false).
let float = minimal_lexical::parse_float::<f64>(12345, -4, false);
println!("float={:?}", float);    // 1.235
```

# License

Lexical is dual licensed under the Apache 2.0 license as well as the MIT license. See the LICENCE-MIT and the LICENCE-APACHE files for the licenses. 

# Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in lexical by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
