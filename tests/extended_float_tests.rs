use minimal_lexical::extended_float;

#[test]
fn moderate_path_test() {
    let (f, valid) = extended_float::moderate_path::<f64>(1234567890, -1, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.0);

    let (f, valid) = extended_float::moderate_path::<f64>(1234567891, -1, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.1);

    let (f, valid) = extended_float::moderate_path::<f64>(12345678912, -2, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.12);

    let (f, valid) = extended_float::moderate_path::<f64>(123456789123, -3, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.123);

    let (f, valid) = extended_float::moderate_path::<f64>(1234567891234, -4, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.1234);

    let (f, valid) = extended_float::moderate_path::<f64>(12345678912345, -5, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.12345);

    let (f, valid) = extended_float::moderate_path::<f64>(123456789123456, -6, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.123456);

    let (f, valid) = extended_float::moderate_path::<f64>(1234567891234567, -7, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.1234567);

    let (f, valid) = extended_float::moderate_path::<f64>(12345678912345679, -8, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 123456789.12345679);

    let (f, valid) = extended_float::moderate_path::<f64>(4628372940652459, -17, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 0.04628372940652459);

    let (f, valid) = extended_float::moderate_path::<f64>(26383446160308229, -272, false);
    assert!(valid, "should be valid");
    assert_eq!(f, 2.6383446160308229e-256);

    let (_, valid) = extended_float::moderate_path::<f64>(26383446160308230, -272, false);
    assert!(!valid, "should be invalid");
}
