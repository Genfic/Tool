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
