minimal-lexical
===============

[![Build Status](https://api.travis-ci.org/Alexhuszagh/minimal-lexical.svg?branch=master)](https://travis-ci.org/Alexhuszagh/minimal-lexical)
[![Latest Version](https://img.shields.io/crates/v/minimal-lexical.svg)](https://crates.io/crates/minimal-lexical)
[![Rustc Version 1.31+](https://img.shields.io/badge/rustc-1.31+-lightgray.svg)](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018.html)

This is a minimal version of [rust-lexical](https://github.com/Alexhuszagh/rust-lexical), meant to allow efficient round-trip float parsing. This implements a complete parser, however.

# Getting Started

```rust
extern crate minimal_lexical;

// Let's say we want to parse "1.2345".
// First, we need an external parser to extract the integer digits ("1"),
// the fraction digits ("2345"), and then parse the exponent to a 32-bit
// integer (0). 
// Warning:
// --------
//  Please note that leading zeros must be trimmed from the integer,
//  and trailing zeros must be trimmed from the fraction. This cannot
//  be handled by minimal-lexical, since we accept iterators
let integer = b"1";
let fraction = b"2345";
let float = minimal_lexical::parse_float::<f64>(integer.iter(), fraction.iter(), 0);
println!("float={:?}", float);    // 1.235
```

# Recipes

You may be asking: where is the actual parser? Due to variation in float formats, and the goal of integrating utility for various data-interchange language parsers, such functionality would be beyond the scope of this library.

For example, the following float is valid in Rust strings, but is invalid in JSON or TOML:
```json
1.e7
```

Therefore, to use the library, you need functionality that extracts the significant digits to pass to `create_float`. Please see [simple-example](examples/simple.rs) for a simple, annotated example on how to use minimal-lexical as a parser.

# Minimum Version Support

Minimal-lexical is tested to support Rustc 1.31+, including stable, beta, and nightly. Please report any errors compiling a supported lexical version on a compatible Rustc version.

# License

Minimal-lexical is dual licensed under the Apache 2.0 license as well as the MIT license. See the LICENCE-MIT and the LICENCE-APACHE files for the licenses. 

# Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in minimal-lexical by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
