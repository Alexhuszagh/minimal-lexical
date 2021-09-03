use minimal_lexical::{algorithm, Float};

// TESTS
// -----

#[test]
fn float_fast_path_test() {
    // valid
    let mantissa = (1 << f32::MANTISSA_SIZE) - 1;
    let (min_exp, max_exp) = f32::exponent_limit();
    for exp in min_exp..max_exp + 1 {
        let f = algorithm::fast_path::<f32>(mantissa, exp);
        assert!(f.is_some(), "should be valid {:?}.", (mantissa, exp));
    }

    // Check slightly above valid exponents
    let f = algorithm::fast_path::<f32>(123, 15);
    assert_eq!(f, Some(1.23e+17));

    // Exponent is 1 too high, pushes over the mantissa.
    let f = algorithm::fast_path::<f32>(123, 16);
    assert!(f.is_none());

    // Mantissa is too large, checked_mul should overflow.
    let f = algorithm::fast_path::<f32>(mantissa, 11);
    assert!(f.is_none());

    // invalid mantissa
    #[cfg(feature = "radix")]
    {
        let (_, max_exp) = f64::exponent_limit(3);
        let f = algorithm::fast_path::<f32>(1 << f32::MANTISSA_SIZE, 3, max_exp + 1);
        assert!(f.is_none(), "invalid mantissa");
    }

    // invalid exponents
    let (min_exp, max_exp) = f32::exponent_limit();
    let f = algorithm::fast_path::<f32>(mantissa, min_exp - 1);
    assert!(f.is_none(), "exponent under min_exp");

    let f = algorithm::fast_path::<f32>(mantissa, max_exp + 1);
    assert!(f.is_none(), "exponent above max_exp");
}

#[test]
fn double_fast_path_test() {
    // valid
    let mantissa = (1 << f64::MANTISSA_SIZE) - 1;
    let (min_exp, max_exp) = f64::exponent_limit();
    for exp in min_exp..max_exp + 1 {
        let f = algorithm::fast_path::<f64>(mantissa, exp);
        assert!(f.is_some(), "should be valid {:?}.", (mantissa, exp));
    }

    // invalid exponents
    let (min_exp, max_exp) = f64::exponent_limit();
    let f = algorithm::fast_path::<f64>(mantissa, min_exp - 1);
    assert!(f.is_none(), "exponent under min_exp");

    let f = algorithm::fast_path::<f64>(mantissa, max_exp + 1);
    assert!(f.is_none(), "exponent above max_exp");

    assert_eq!(Some(0.04628372940652459), algorithm::fast_path::<f64>(4628372940652459, -17));
    assert_eq!(None, algorithm::fast_path::<f64>(26383446160308229, -272));
}
