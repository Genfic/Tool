use super::{format_bytes, to_fixed};

#[test]
fn bytes_match_std_fmt_examples() {
    assert_eq!(format_bytes(0.0), "0 B");
    assert_eq!(format_bytes(100.0), "100 B");
    assert_eq!(format_bytes(1337.0), "1.34 kB");
    assert_eq!(format_bytes(1000.0), "1 kB");
    assert_eq!(format_bytes(1_048_576.0), "1.05 MB");
    assert_eq!(format_bytes(999.0), "999 B");
}

#[test]
#[allow(clippy::approx_constant)] // `3.14159` is arbitrary test data, not π.
fn bytes_match_deno_std_fmt_bytes() {
    // Each pair is `[input, format(input)]` produced by running the real
    // `@std/fmt/bytes` implementation under Deno.
    let cases: &[(f64, &str)] = &[
        (0.0, "0 B"),
        (1.0, "1 B"),
        (2.0, "2 B"),
        (10.0, "10 B"),
        (999.0, "999 B"),
        (1000.0, "1 kB"),
        (1001.0, "1 kB"),
        (1023.0, "1.02 kB"),
        (1024.0, "1.02 kB"),
        (1337.0, "1.34 kB"),
        (2048.0, "2.05 kB"),
        (9999.0, "10 kB"),
        (10000.0, "10 kB"),
        (99999.0, "100 kB"),
        (100000.0, "100 kB"),
        (123456.0, "123 kB"),
        (999999.0, "1000 kB"),
        (1000000.0, "1 MB"),
        (1048576.0, "1.05 MB"),
        (1500000.0, "1.5 MB"),
        (9999999.0, "10 MB"),
        (10000000.0, "10 MB"),
        (100000000.0, "100 MB"),
        (1000000000.0, "1 GB"),
        (1234567890.0, "1.23 GB"),
        (1e12, "1 TB"),
        (1e15, "1 PB"),
        (1e18, "1 EB"),
        (1e21, "1 ZB"),
        (1e24, "1 YB"),
        (0.5, "0.5 B"),
        (0.999, "0.999 B"),
        (0.001, "0.001 B"),
        (1.001, "1 B"),
        (1.5, "1.5 B"),
        (2.5, "2.5 B"),
        (3.14159, "3.14 B"),
        (45.6749, "45.7 B"),
        (100.5, "101 B"),
        (999.99, "1000 B"),
        (1000.5, "1 kB"),
        (1001.9, "1 kB"),
        (1e-6, "0.000001 B"),
        (-1.0, "-1 B"),
        (-100.0, "-100 B"),
        (-1337.0, "-1.34 kB"),
        (-1048576.0, "-1.05 MB"),
        (-1000000000.0, "-1 GB"),
        (-0.5, "-0.5 B"),
        (9007199254740991.0, "9.01 PB"),
        (-9007199254740991.0, "-9.01 PB"),
        (f64::MAX, "1.8e+284 YB"),
        (5e-324, "5e-324 B"),
    ];
    for &(input, expected) in cases {
        assert_eq!(format_bytes(input), expected, "format_bytes({input})");
    }
}

#[test]
fn to_fixed_matches_javascript() {
    // Values and expectations verified against Node's `Number#toFixed`.
    assert_eq!(to_fixed(45.6749, 2), "45.67");
    assert_eq!(to_fixed(0.125, 2), "0.13");
    assert_eq!(to_fixed(100.0, 2), "100.00");
    assert_eq!(to_fixed(0.0, 2), "0.00");
    // `2.005` is stored just below `2.005`, exactly like JavaScript's
    // `(2.005).toFixed(2) === "2.00"`.
    assert_eq!(to_fixed(2.005, 2), "2.00");
    assert_eq!(to_fixed(1.005, 2), "1.00");
    assert_eq!(to_fixed(2.675, 2), "2.67");
    assert_eq!(to_fixed(9.995, 2), "9.99");
    assert_eq!(to_fixed(1.255, 2), "1.25");
    assert_eq!(to_fixed(0.615, 2), "0.61");
    assert_eq!(to_fixed(1234.5678, 3), "1234.568");
    assert_eq!(to_fixed(9.999, 2), "10.00");
    assert_eq!(to_fixed(1.5, 0), "2");
    assert_eq!(to_fixed(0.0, 0), "0");
    // A negative value keeps its sign even when it rounds to zero.
    assert_eq!(to_fixed(-0.004, 2), "-0.00");
    assert_eq!(to_fixed(-2.005, 2), "-2.00");
    assert_eq!(to_fixed(-0.5, 0), "-1");
    assert_eq!(to_fixed(-0.4, 0), "-0");
    // `-0` is not less than zero, so it has no sign.
    assert_eq!(to_fixed(-0.0, 2), "0.00");
    // Non-finite values pass through.
    assert_eq!(to_fixed(f64::NAN, 2), "NaN");
    assert_eq!(to_fixed(f64::INFINITY, 2), "Infinity");
    assert_eq!(to_fixed(f64::NEG_INFINITY, 2), "-Infinity");
}
