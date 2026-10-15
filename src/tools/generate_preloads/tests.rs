use super::render_preloads;

#[test]
fn renders_preload_tags_and_normalises_the_destination() {
    assert_eq!(
        render_preloads(&["a.woff2".to_string(), "b.woff".to_string()], "/fonts"),
        vec![
            "<link rel=\"preload\" as=\"font\" crossorigin=\"anonymous\" href=\"~/fonts/a.woff2\">",
            "<link rel=\"preload\" as=\"font\" crossorigin=\"anonymous\" href=\"~/fonts/b.woff\">",
        ]
    );
}

#[test]
fn keeps_the_given_order() {
    assert_eq!(
        render_preloads(&["z.woff2".to_string(), "a.woff2".to_string()], "fonts"),
        vec![
            "<link rel=\"preload\" as=\"font\" crossorigin=\"anonymous\" href=\"~/fonts/z.woff2\">",
            "<link rel=\"preload\" as=\"font\" crossorigin=\"anonymous\" href=\"~/fonts/a.woff2\">",
        ]
    );
}
