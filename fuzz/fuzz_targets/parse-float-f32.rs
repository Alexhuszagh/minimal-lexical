#![no_main]

mod parse;

#[macro_use]
extern crate libfuzzer_sys;

fuzz_target!(|data: &[u8]| {
    let _ = parse::parse_float::<f32>(data);
});
