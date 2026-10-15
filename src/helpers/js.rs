//! Helpers that reproduce specific JavaScript / es-toolkit behaviours used by
//! the original tools.

use std::cmp::Ordering;

/// Splits a string into words the way es-toolkit's `words` does.
///
/// es-toolkit builds this from the regular expression
/// `\p{Lu}?\p{Ll}+|[0-9]+|\p{Lu}+(?!\p{Ll})|\p{Emoji_Presentation}|\p{Extended_Pictographic}|\p{L}+`.
/// The Rust `regex` crate does not support the emoji properties nor the
/// negative look-ahead, so the equivalent scan is implemented by hand.
///
/// Difference from es-toolkit: emoji/other symbols are treated as separators
/// instead of standalone words. OpenAPI identifiers never contain emoji, so
/// the difference is not observable for this tool.
pub fn split_words(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut words = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let current = chars[i];

        // \p{Lu}?\p{Ll}+ — a run of lowercase letters, optionally preceded by a
        // single uppercase letter.
        if current.is_lowercase()
            || (current.is_uppercase() && i + 1 < chars.len() && chars[i + 1].is_lowercase())
        {
            let start = i;
            if current.is_uppercase() {
                i += 1;
            }
            while i < chars.len() && chars[i].is_lowercase() {
                i += 1;
            }
            words.push(chars[start..i].iter().collect());
            continue;
        }

        // [0-9]+
        if current.is_ascii_digit() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            words.push(chars[start..i].iter().collect());
            continue;
        }

        // \p{Lu}+(?!\p{Ll})  (greedy, backtracks by one when followed by a
        // lowercase letter so the acronym boundary is preserved)
        if current.is_uppercase() {
            let start = i;
            while i < chars.len() && chars[i].is_uppercase() {
                i += 1;
            }
            if i < chars.len() && chars[i].is_lowercase() && i - start > 1 {
                i -= 1;
            }
            words.push(chars[start..i].iter().collect());
            continue;
        }

        // \p{L}+
        if current.is_alphabetic() {
            let start = i;
            while i < chars.len() && chars[i].is_alphabetic() {
                i += 1;
            }
            words.push(chars[start..i].iter().collect());
            continue;
        }

        // Anything else is a separator.
        i += 1;
    }

    words
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out: String = first.to_uppercase().collect();
            out.push_str(&chars.as_str().to_lowercase());
            out
        }
    }
}

/// Port of es-toolkit's `camelCase`.
pub fn camel_case(input: &str) -> String {
    let mut words = split_words(input);
    if words.is_empty() {
        return String::new();
    }

    let first = words.remove(0);
    let mut out = first.to_lowercase();
    for word in words {
        out.push_str(&capitalize(&word));
    }
    out
}

/// Port of es-toolkit's `startCase`.
pub fn start_case(input: &str) -> String {
    let mut result = String::new();
    for word in split_words(input.trim()) {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&capitalize(&word));
    }
    result
}

/// `String(value)` for the value shapes that can appear inside an OpenAPI
/// `enum` array.
pub fn js_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        other => other.to_string(),
    }
}

/// Approximation of `String.prototype.localeCompare` for ASCII identifiers.
///
/// JavaScript's locale aware comparison orders case-insensitively at the
/// primary level (`"number".localeCompare("Url") > 0`), which plain byte
/// comparison gets wrong for mixed-case unions. Comparisons fall back to the
/// raw byte order for the (rare) case-insensitive ties.
pub fn locale_compare(a: &str, b: &str) -> Ordering {
    let lower_a = a.to_lowercase();
    let lower_b = b.to_lowercase();
    lower_a.cmp(&lower_b).then_with(|| a.cmp(b))
}

/// Port of `encodeURIComponent`.
pub fn encode_uri_component(input: &str) -> String {
    const UNRESERVED: &str =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_.!~*'()";
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        if (byte as char).is_ascii() && UNRESERVED.contains(byte as char) {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{camel_case, encode_uri_component, locale_compare, split_words, start_case};
    use std::cmp::Ordering;

    #[test]
    fn words_match_es_toolkit() {
        assert_eq!(
            split_words("fred, barney, & pebbles"),
            ["fred", "barney", "pebbles"]
        );
        assert_eq!(
            split_words("camelCaseHTTPRequest"),
            ["camel", "Case", "HTTP", "Request"]
        );
        assert_eq!(split_words("hello123world"), ["hello", "123", "world"]);
        assert_eq!(
            split_words("XMLHttpRequest2.0_parser-v1.2"),
            ["XML", "Http", "Request", "2", "0", "parser", "v", "1", "2"]
        );
        assert_eq!(split_words("Lunedì 18 Set"), ["Lunedì", "18", "Set"]);
    }

    #[test]
    fn camel_case_matches_es_toolkit() {
        assert_eq!(camel_case("camelCase"), "camelCase");
        assert_eq!(camel_case("some whitespace"), "someWhitespace");
        assert_eq!(camel_case("hyphen-text"), "hyphenText");
        assert_eq!(camel_case("HTTPRequest"), "httpRequest");
        assert_eq!(camel_case("__FOO_BAR__"), "fooBar");
        assert_eq!(camel_case("user-id"), "userId");
        assert_eq!(camel_case(""), "");
    }

    #[test]
    fn start_case_matches_es_toolkit() {
        assert_eq!(start_case("hello world"), "Hello World");
        assert_eq!(start_case("HELLO WORLD"), "Hello World");
        assert_eq!(start_case("hello-world"), "Hello World");
        assert_eq!(start_case("hello_world"), "Hello World");
        assert_eq!(start_case("name"), "Name");
        assert_eq!(start_case("size"), "Size");
    }

    #[test]
    fn locale_compare_orders_case_insensitively_first() {
        // These values were checked against `String.prototype.localeCompare`
        // in the ICU default locale with Node.
        assert_eq!(locale_compare("number", "Url"), Ordering::Less);
        assert_eq!(locale_compare("Url", "number"), Ordering::Greater);
        assert_eq!(locale_compare("Foo", "string"), Ordering::Less);
        assert_eq!(locale_compare("Date", "number"), Ordering::Less);
        assert_eq!(locale_compare("number", "Date"), Ordering::Greater);
    }

    #[test]
    fn encodes_uri_components_like_javascript() {
        assert_eq!(
            encode_uri_component("https://localhost:5001/openapi/public.json"),
            "https%3A%2F%2Flocalhost%3A5001%2Fopenapi%2Fpublic.json"
        );
        assert_eq!(
            encode_uri_component("a-b_c.d!e~f*g'h(i)"),
            "a-b_c.d!e~f*g'h(i)"
        );
        assert_eq!(encode_uri_component("ä"), "%C3%A4");
    }
}
