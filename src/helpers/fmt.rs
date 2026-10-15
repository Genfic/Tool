//! Number formatting helpers that mirror the JavaScript behaviour the Deno
//! tools relied on (`@std/fmt/bytes` and `Number.prototype.toFixed`).

/// Renders a byte count the way `@std/fmt/bytes`'s `format` does with no
/// options: SI units, three significant figures, e.g. `1337` -> `1.34 kB`.
pub fn format_bytes(num: f64) -> String {
    const UNITS_FIRST_LETTER: &str = "BkMGTPEZY";

    let prefix = if num < 0.0 { "-" } else { "" };
    let num = num.abs();

    if num < 1.0 {
        return format!("{prefix}{} B", js_number_to_string(num));
    }

    let exponent = (num.log10() / 3.0)
        .floor()
        .min((UNITS_FIRST_LETTER.len() - 1) as f64) as usize;
    let mut num = num / 1000_f64.powi(exponent as i32);

    // `Number(num.toPrecision(3))`
    num = to_precision_3(num);

    let mut unit = UNITS_FIRST_LETTER[exponent..exponent + 1].to_string();
    if exponent > 0 {
        unit.push('B');
    }

    format!("{prefix}{} {unit}", js_number_to_string(num))
}

/// Number of fraction digits requested from Rust's float formatter when taking
/// an exact snapshot of a `f64`. The exact decimal expansion of a finite `f64`
/// never has more than 1074 fraction digits, so this is comfortably exact.
const EXACT_DIGITS: usize = 1100;

/// Formats a number with exactly `digits` fraction digits, matching
/// `Number.prototype.toFixed`.
///
/// JavaScript rounds the *exact* value of the double (not its shortest decimal
/// spelling) and breaks ties toward `+Infinity` for the absolute value. So
/// `(2.005).toFixed(2) === "2.00"` because `2.005` is stored slightly below
/// `2.005`, while `(0.125).toFixed(2) === "0.13"` because a tie rounds up.
/// A negative input keeps its sign even when it rounds to zero (`-0.004` ->
/// `-0.00`), but `-0` itself does not (`(-0).toFixed(2) === "0.00"`).
pub fn to_fixed(value: f64, digits: usize) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }

    let negative = value < 0.0;
    // Exact decimal expansion of the absolute value, e.g. `2.005` becomes
    // `2.0049999999999998934185896359849721193313598632812500...`.
    let exact = format!("{:.precision$}", value.abs(), precision = EXACT_DIGITS);
    let (int_part, frac_part) = exact.split_once('.').unwrap_or((exact.as_str(), ""));

    let kept = &frac_part[..digits.min(frac_part.len())];
    // Round up whenever the first dropped digit is 5 or more; an exact half is
    // a tie and JavaScript breaks it upward.
    let round_up = frac_part.as_bytes().get(digits).is_some_and(|&b| b >= b'5');

    let mut combined = String::with_capacity(int_part.len() + kept.len() + 1);
    combined.push_str(int_part);
    combined.push_str(kept);
    if round_up {
        combined = increment_decimal(&combined);
    }

    let int_len = combined.len().saturating_sub(digits);
    let mut out = String::with_capacity(combined.len() + 1);
    if negative {
        out.push('-');
    }
    if int_len == 0 {
        out.push('0');
    } else {
        out.push_str(&combined[..int_len]);
    }
    if digits > 0 {
        out.push('.');
        if int_len == 0 {
            // Pad with leading zeros for values below one.
            for _ in 0..(digits - combined.len()) {
                out.push('0');
            }
        }
        out.push_str(&combined[int_len..]);
    }
    out
}

/// Adds one to a non-negative decimal string, growing it if it carries over.
fn increment_decimal(digits: &str) -> String {
    let mut bytes = digits.as_bytes().to_vec();
    for i in (0..bytes.len()).rev() {
        if bytes[i] == b'9' {
            bytes[i] = b'0';
        } else {
            bytes[i] += 1;
            return String::from_utf8(bytes).expect("only ASCII digits");
        }
    }
    let mut out = String::with_capacity(bytes.len() + 1);
    out.push('1');
    out.push_str(std::str::from_utf8(&bytes).expect("only ASCII digits"));
    out
}

/// `Number#toString` for base 10, following the ECMAScript `Number::toString`
/// algorithm. Rust's `Display` for `f64` prints the shortest round-tripping
/// digits but never uses exponential notation, whereas JavaScript switches to
/// exponential form for `|x| >= 1e21` and `|x| < 1e-6`.
pub fn js_number_to_string(value: f64) -> String {
    if value == 0.0 {
        // Also normalises `-0`.
        return "0".to_string();
    }
    if !value.is_finite() {
        return if value.is_nan() {
            "NaN".to_string()
        } else if value > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }

    let negative = value < 0.0;
    // `{:e}` gives the shortest round-tripping significant digits as
    // `d[.ddd]e<exp>`, which is exactly the `digits`/`n` pair ECMAScript needs.
    let scientific = format!("{:e}", value.abs());
    let (mantissa, exponent) = scientific
        .split_once('e')
        .expect("LowerExp always emits an exponent");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let exponent: i32 = exponent.parse().expect("LowerExp emits a decimal exponent");
    let k = digits.len() as i32;
    // `value = 0.<digits> * 10^n`.
    let n = exponent + 1;

    let mut out = String::new();
    if k <= n && n <= 21 {
        // Integer with trailing zeros.
        out.push_str(&digits);
        for _ in 0..(n - k) {
            out.push('0');
        }
    } else if 0 < n && n <= 21 {
        // Decimal point inside the digits.
        out.push_str(&digits[..n as usize]);
        out.push('.');
        out.push_str(&digits[n as usize..]);
    } else if -6 < n && n <= 0 {
        // Leading `0.` and zeros.
        out.push_str("0.");
        for _ in 0..(-n) {
            out.push('0');
        }
        out.push_str(&digits);
    } else {
        // Exponential form.
        if k == 1 {
            out.push_str(&digits);
        } else {
            out.push_str(&digits[..1]);
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push('e');
        if n > 0 {
            out.push('+');
        }
        out.push_str(&(n - 1).to_string());
    }

    if negative {
        out.insert(0, '-');
    }
    out
}

/// `Number(value.toPrecision(3))` for the values the unit conversion in
/// [`format_bytes`] produces.
fn to_precision_3(x: f64) -> f64 {
    js_to_precision(x, 3).parse::<f64>().unwrap_or(x)
}

/// `Number#toPrecision(p)` for a finite value, following the ECMAScript
/// algorithm. Like `to_fixed`, it rounds the exact double value half-up rather
/// than its shortest decimal spelling.
fn js_to_precision(value: f64, p: usize) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if value == 0.0 {
        return if p == 0 {
            "0".to_string()
        } else {
            format!("0.{}", "0".repeat(p - 1))
        };
    }

    let negative = value < 0.0;
    let exact = format!("{:.precision$}", value.abs(), precision = EXACT_DIGITS);
    let (int_part, frac_part) = exact.split_once('.').unwrap_or((exact.as_str(), ""));

    let mut digits = Vec::with_capacity(int_part.len() + frac_part.len());
    digits.extend_from_slice(int_part.as_bytes());
    digits.extend_from_slice(frac_part.as_bytes());

    let first = digits
        .iter()
        .position(|&b| b != b'0')
        .expect("a non-zero value has a non-zero digit");
    // Decimal exponent `e`: `value = 0.<digits[first..]> * 10^(e + 1)`.
    let mut e = int_part.len() as i64 - 1 - first as i64;

    let end = (first + p).min(digits.len());
    let mut significant = digits[first..end].to_vec();
    significant.resize(p, b'0');

    if digits.get(first + p).is_some_and(|&b| b >= b'5') {
        let mut carry = true;
        for byte in significant.iter_mut().rev() {
            if *byte == b'9' {
                *byte = b'0';
            } else {
                *byte += 1;
                carry = false;
                break;
            }
        }
        if carry {
            // All nines overflowed to `1` followed by zeros.
            e += 1;
            significant.clear();
            significant.push(b'1');
            significant.resize(p, b'0');
        }
    }

    let significant = String::from_utf8(significant).expect("ASCII digits");
    let mut out = String::with_capacity(significant.len() + 8);
    if negative {
        out.push('-');
    }
    if e < -6 || e >= p as i64 {
        out.push(significant.as_bytes()[0] as char);
        if p > 1 {
            out.push('.');
            out.push_str(&significant[1..]);
        }
        out.push('e');
        if e >= 0 {
            out.push('+');
        }
        out.push_str(&e.to_string());
    } else if e == p as i64 - 1 {
        out.push_str(&significant);
    } else if e >= 0 {
        out.push_str(&significant[..(e + 1) as usize]);
        out.push('.');
        out.push_str(&significant[(e + 1) as usize..]);
    } else {
        out.push_str("0.");
        for _ in 0..(-(e + 1)) {
            out.push('0');
        }
        out.push_str(&significant);
    }
    out
}

#[cfg(test)]
mod tests {
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
}
