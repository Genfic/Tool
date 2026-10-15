//! Port of `tools/generate-imports/index.ts`.
//!
//! `generateImports` walks `source` and prints a CSS source list for every font
//! file it finds. Files are ordered by their extension priority (`woff2`,
//! `woff`, `otf`, `ttf`, then everything else) and then by name, descending.

use std::fs;

use anyhow::{Context, Result};

use crate::helpers::string_utils::trim_destination;

/// Extension priority table from the original (`priorities`). Unknown
/// extensions fall back to `0`.
fn priority(extension: Option<&str>) -> u32 {
    match extension {
        Some("woff2") => 1,
        Some("woff") => 2,
        Some("otf") => 3,
        Some("ttf") => 4,
        _ => 0,
    }
}

/// `name.split(".")[1]`, i.e. the text between the first and second dot.
fn second_segment(name: &str) -> Option<&str> {
    name.split('.').nth(1)
}

/// Sorts the file names exactly like the original and renders one line per
/// file.
pub fn render_imports(names: &[String], dest: &str) -> Vec<String> {
    let directory = trim_destination(dest);

    let mut tags: Vec<(&String, u32)> = names
        .iter()
        .map(|name| (name, priority(second_segment(name))))
        .collect();

    // Stable sort by priority, then stable sort by name, then reverse. Because
    // tag names are unique the result is simply the names in descending order,
    // but the original keeps both sorts.
    tags.sort_by_key(|(_, priority)| *priority);
    tags.sort_by(|a, b| a.0.cmp(b.0));
    tags.reverse();

    tags.iter()
        .map(|(tag, _)| {
            let extension = second_segment(tag).unwrap_or("undefined");
            format!("url('/{directory}/{tag}') format('{extension}'),")
        })
        .collect()
}

/// Port of `generateImports`.
pub fn generate_imports(source: &str, dest: &str) -> Result<()> {
    // `Deno.readDir(source)` yields directory entries in OS order; the sorts in
    // `render_imports` make the final order deterministic.
    let entries =
        fs::read_dir(source).with_context(|| format!("Failed to read directory {source}"))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("Failed to read entry in {source}"))?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }

    for line in render_imports(&names, dest) {
        println!("{line}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
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
}
