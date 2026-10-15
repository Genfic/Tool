use super::{expand_braces, percentage};

#[test]
fn expands_brace_groups() {
    assert_eq!(
        expand_braces("js/{dist,bundle}/**/*.js"),
        ["js/dist/**/*.js", "js/bundle/**/*.js"]
    );
    assert_eq!(expand_braces("{a,b}/{c,d}"), ["a/c", "a/d", "b/c", "b/d"]);
}

#[test]
fn leaves_brace_free_patterns_untouched() {
    assert_eq!(expand_braces("js/**/*.js"), ["js/**/*.js"]);
    // A group without a comma matches the same text as a literal.
    assert_eq!(expand_braces("js/{dist}/a.js"), ["js/dist/a.js"]);
}

#[test]
fn keeps_unbalanced_braces_literal() {
    assert_eq!(expand_braces("js/{dist,a.js"), ["js/{dist,a.js"]);
}

#[test]
fn percentage_uses_two_fixed_digits() {
    assert_eq!(percentage(1, 4), "25.00");
    assert_eq!(percentage(1, 3), "33.33");
    assert_eq!(percentage(0, 4), "0.00");
}
