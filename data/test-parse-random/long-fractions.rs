// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod _common;

use self::_common::{parse_float, validate};
use std::char;

fn main() {
    for n in 0..10 {
        let digit = char::from_digit(n, 10).unwrap();
        let mut s = "0.".to_string();
        for _ in 0..400 {
            s.push(digit);
            if parse_float::<f64>(s.as_bytes()).1.len() == 0 {
                validate(&s);
            }
        }
    }
}
