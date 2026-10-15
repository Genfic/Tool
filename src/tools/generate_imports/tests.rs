use super::{priority, render_imports, second_segment};

#[test]
fn priorities_follow_the_original_table() {
    assert_eq!(priority(Some("woff2")), 1);
    assert_eq!(priority(Some("woff")), 2);
    assert_eq!(priority(Some("otf")), 3);
    assert_eq!(priority(Some("ttf")), 4);
    assert_eq!(priority(Some("eot")), 0);
    assert_eq!(priority(None), 0);
}

#[test]
fn second_segment_matches_javascript_indexing() {
    assert_eq!(second_segment("roboto-latin-400.woff2"), Some("woff2"));
    assert_eq!(second_segment("a.b.woff"), Some("b"));
    assert_eq!(second_segment(".woff2"), Some("woff2"));
    assert_eq!(second_segment("no-extension"), None);
}

#[test]
fn lists_files_in_descending_name_order_and_normalises_the_destination() {
    let names = vec![
        "b-font.woff".to_string(),
        "a-font.woff2".to_string(),
        "c-font.ttf".to_string(),
        "d-font.otf".to_string(),
    ];
    assert_eq!(
        render_imports(&names, "/fonts/"),
        vec![
            "url('/fonts/d-font.otf') format('otf'),",
            "url('/fonts/c-font.ttf') format('ttf'),",
            "url('/fonts/b-font.woff') format('woff'),",
            "url('/fonts/a-font.woff2') format('woff2'),",
        ]
    );
}

#[test]
fn formats_extension_less_files_like_javascript() {
    assert_eq!(
        render_imports(&["LICENSE".to_string()], "fonts"),
        vec!["url('/fonts/LICENSE') format('undefined'),"]
    );
}
