use minimal_lexical::lemire;

#[test]
fn test_halfway_round_down() {
    // Check only Eisel-Lemire.
    assert_eq!((9007199254740992.0, true), lemire::eisel_lemire::<f64>(9007199254740992, 0));
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9007199254740993, 0));
    assert_eq!((9007199254740994.0, true), lemire::eisel_lemire::<f64>(9007199254740994, 0));
    assert_eq!((9223372036854775808.0, true), lemire::eisel_lemire::<f64>(9223372036854775808, 0));
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9223372036854776832, 0));
    assert_eq!((9223372036854777856.0, true), lemire::eisel_lemire::<f64>(9223372036854777856, 0));

    // We can't get an accurate representation here.
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9007199254740992000, -3));
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9007199254740993000, -3));
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9007199254740994000, -3));

    // Check with the extended-float backup.
    assert_eq!(
        (9007199254740992.0, true),
        lemire::moderate_path::<f64>(9007199254740992, 0, false)
    );
    assert_eq!(
        (9007199254740992.0, false),
        lemire::moderate_path::<f64>(9007199254740993, 0, false)
    );
    assert_eq!(
        (9007199254740994.0, true),
        lemire::moderate_path::<f64>(9007199254740994, 0, false)
    );
    assert_eq!(
        (9223372036854775808.0, true),
        lemire::moderate_path::<f64>(9223372036854775808, 0, false)
    );
    assert_eq!(
        (9223372036854775808.0, false),
        lemire::moderate_path::<f64>(9223372036854776832, 0, false)
    );
    assert_eq!(
        (9223372036854777856.0, true),
        lemire::moderate_path::<f64>(9223372036854777856, 0, false)
    );

    // We can't get an accurate from Lemire representation here.
    assert_eq!(
        (9007199254740992.0, true),
        lemire::moderate_path::<f64>(9007199254740992000, -3, false)
    );
    assert_eq!(
        (9007199254740992.0, false),
        lemire::moderate_path::<f64>(9007199254740993000, -3, false)
    );
    assert_eq!(
        (9007199254740994.0, true),
        lemire::moderate_path::<f64>(9007199254740994000, -3, false)
    );
}

#[test]
fn test_halfway_round_up() {
    // Check only Eisel-Lemire.
    assert_eq!((9007199254740994.0, true), lemire::eisel_lemire::<f64>(9007199254740994, 0));
    assert_eq!((9007199254740996.0, true), lemire::eisel_lemire::<f64>(9007199254740995, 0));
    assert_eq!((9007199254740996.0, true), lemire::eisel_lemire::<f64>(9007199254740996, 0));
    assert_eq!((18014398509481988.0, true), lemire::eisel_lemire::<f64>(18014398509481988, 0));
    assert_eq!((18014398509481992.0, true), lemire::eisel_lemire::<f64>(18014398509481990, 0));
    assert_eq!((18014398509481992.0, true), lemire::eisel_lemire::<f64>(18014398509481992, 0));
    assert_eq!((9223372036854777856.0, true), lemire::eisel_lemire::<f64>(9223372036854777856, 0));
    assert_eq!((9223372036854779904.0, true), lemire::eisel_lemire::<f64>(9223372036854778880, 0));
    assert_eq!((9223372036854779904.0, true), lemire::eisel_lemire::<f64>(9223372036854779904, 0));

    // We can't get an accurate representation here.
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9007199254740994000, -3));
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9007199254740995000, -3));
    assert_eq!((0.0, false), lemire::eisel_lemire::<f64>(9007199254740996000, -3));

    // Check with the extended-float backup.
    assert_eq!(
        (9007199254740994.0, true),
        lemire::moderate_path::<f64>(9007199254740994, 0, false)
    );
    assert_eq!(
        (9007199254740996.0, true),
        lemire::moderate_path::<f64>(9007199254740995, 0, false)
    );
    assert_eq!(
        (9007199254740996.0, true),
        lemire::moderate_path::<f64>(9007199254740996, 0, false)
    );
    assert_eq!(
        (18014398509481988.0, true),
        lemire::moderate_path::<f64>(18014398509481988, 0, false)
    );
    assert_eq!(
        (18014398509481992.0, true),
        lemire::moderate_path::<f64>(18014398509481990, 0, false)
    );
    assert_eq!(
        (18014398509481992.0, true),
        lemire::moderate_path::<f64>(18014398509481992, 0, false)
    );
    assert_eq!(
        (9223372036854777856.0, true),
        lemire::moderate_path::<f64>(9223372036854777856, 0, false)
    );
    assert_eq!(
        (9223372036854779904.0, true),
        lemire::moderate_path::<f64>(9223372036854778880, 0, false)
    );
    assert_eq!(
        (9223372036854779904.0, true),
        lemire::moderate_path::<f64>(9223372036854779904, 0, false)
    );

    // We can't get an accurate from Lemire representation here.
    assert_eq!(
        (9007199254740994.0, true),
        lemire::moderate_path::<f64>(9007199254740994000, -3, false)
    );
    assert_eq!(
        (9007199254740994.0, false),
        lemire::moderate_path::<f64>(9007199254740995000, -3, false)
    );
    assert_eq!(
        (9007199254740996.0, true),
        lemire::moderate_path::<f64>(9007199254740996000, -3, false)
    );
}

#[test]
fn test_mul() {
    let e1 = 11529215046068469760; // 1e1
    let e10 = 10737418240000000000; // 1e10
    assert_eq!((0x5D21DBA000000000, 0x0000000000000000), lemire::mul(e1, e10));

    let e9 = 17179869184000000000; // 1e9
    let e70 = 13363823550460978230; // 1e70
    assert_eq!((0xACB92ED9397BF995, 0xA23A700000000000), lemire::mul(e9, e70));

    // e289
    let e280 = 10162340898095201970; // 1e280
    assert_eq!((0x83585D8FD9C25DB6, 0xFC31D00000000000), lemire::mul(e9, e280));

    // e290
    let e0 = 9223372036854775808; // 1e0
    let e290 = 11830521861667747109; // 1e290
    assert_eq!((0x52173A79E8197A92, 0x8000000000000000), lemire::mul(e0, e290));
}
