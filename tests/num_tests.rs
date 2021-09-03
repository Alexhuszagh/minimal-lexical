use minimal_lexical::num;

fn check_as_primitive<T: num::AsPrimitive>(t: T) {
    let _: u64 = t.as_u64();
}

#[test]
fn as_primitive_test() {
    check_as_primitive(1u64);
}

fn check_number<T: num::Number>(x: T, mut y: T) {
    // Copy, partialeq, partialord
    let _ = x;
    assert!(x < y);
    assert!(x != y);

    // Operations
    let _ = y + x;
    let _ = y - x;
    let _ = y * x;
    let _ = y / x;
    let _ = y % x;
    y += x;
    y -= x;
    y *= x;
    y /= x;
    y %= x;

    // Conversions already tested.
}

#[test]
fn number_test() {
    check_number(1u64, 5);
}

fn check_integer<T: num::Integer>(x: T) {
    // Bitwise operations
    let _ = x & T::ZERO;
}

#[test]
fn integer_test() {
    check_integer(65u64);
}

fn check_float<T: num::Float>(x: T) {
    // Check functions
    let _ = x.pow10(5);
    let _ = x.to_bits();
    assert!(T::from_bits(x.to_bits()) == x);

    // Check properties
    let _ = x.to_bits() & T::SIGN_MASK;
    let _ = x.to_bits() & T::EXPONENT_MASK;
    let _ = x.to_bits() & T::HIDDEN_BIT_MASK;
    let _ = x.to_bits() & T::MANTISSA_MASK;
}

#[test]
fn float_test() {
    check_float(123f32);
    check_float(123f64);
}
