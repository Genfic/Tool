//! Port of `tools/bundlesize/index.ts`.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use glob::glob;

use crate::helpers::fmt::{format_bytes, to_fixed};
use crate::helpers::table::Table;

struct PathData {
    name: String,
    path: PathBuf,
}

/// Port of `bundleSize`.
pub fn bundle_size(patterns: &[String]) -> Result<()> {
    let mut groups: Vec<(String, Vec<PathData>)> = Vec::new();
    let mut index: HashMap<String, usize> = HashMap::new();

    for pattern in patterns {
        for expanded in expand_braces(pattern) {
            let entries =
                glob(&expanded).with_context(|| format!("Invalid glob pattern: {expanded}"))?;
            for entry in entries {
                let path =
                    entry.with_context(|| format!("Failed to read glob match for: {expanded}"))?;
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let mut dir = path
                    .parent()
                    .map(|parent| parent.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if dir.is_empty() {
                    // `path.dirname("a.js") === "."`
                    dir = ".".to_string();
                }

                let data = PathData { name, path };
                match index.get(&dir) {
                    Some(position) => groups[*position].1.push(data),
                    None => {
                        index.insert(dir.clone(), groups.len());
                        groups.push((dir, vec![data]));
                    }
                }
            }
        }
    }

    let mut plain_total: u64 = 0;
    let mut brotli_total: u64 = 0;
    let mut gzip_total: u64 = 0;
    let mut zstd_total: u64 = 0;

    for (dir, files) in &groups {
        println!("{dir}");

        let mut plain: Vec<(String, u64)> = Vec::with_capacity(files.len());
        for file in files {
            let size = fs::metadata(&file.path)
                .with_context(|| format!("Failed to stat {}", file.path.display()))?
                .len();
            plain.push((file.name.clone(), size));
        }
        plain.sort_by_key(|a| std::cmp::Reverse(a.1));

        let total: u64 = plain.iter().map(|(_, size)| size).sum();
        let (brotli, gzip, zstd) = compressed_sizes(files)?;

        plain_total += total;
        brotli_total += brotli;
        gzip_total += gzip;
        zstd_total += zstd;

        let table = Table::new()
            .object_body(
                vec!["name".to_string(), "size".to_string()],
                plain
                    .iter()
                    .map(|(name, size)| vec![name.clone(), format_bytes(*size as f64)])
                    .collect(),
            )
            .footer(vec![
                vec!["Total size".to_string(), format_bytes(total as f64)],
                vec![
                    "Gzip".to_string(),
                    format!(
                        "{} ({}%)",
                        format_bytes(gzip as f64),
                        percentage(gzip, total)
                    ),
                ],
                vec![
                    "Brotli".to_string(),
                    format!(
                        "{} ({}%)",
                        format_bytes(brotli as f64),
                        percentage(brotli, total)
                    ),
                ],
                vec![
                    "Zstd".to_string(),
                    format!(
                        "{} ({}%)",
                        format_bytes(zstd as f64),
                        percentage(zstd, total)
                    ),
                ],
            ]);

        println!("{}", table.render());
    }

    println!(
        "Total JS size: {} ({} gzipped, {} brotli, {} zstd)",
        format_bytes(plain_total as f64),
        format_bytes(gzip_total as f64),
        format_bytes(brotli_total as f64),
        format_bytes(zstd_total as f64),
    );

    Ok(())
}

fn percentage(part: u64, total: u64) -> String {
    let percent = (part as f64 / total as f64) * 100.0;
    to_fixed(percent, 2)
}

/// Port of `getCompressedSizes`: the file contents are concatenated as text
/// and compressed as a single blob.
fn compressed_sizes(files: &[PathData]) -> Result<(u64, u64, u64)> {
    let mut concatenated = String::new();
    for file in files {
        concatenated.push_str(
            &fs::read_to_string(&file.path)
                .with_context(|| format!("Failed to read {}", file.path.display()))?,
        );
    }
    let content = concatenated.as_bytes();

    let brotli = {
        let mut output = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut output, 4096, 11, 22);
            writer.write_all(content)?;
        }
        output.len()
    };

    let gzip = {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(content)?;
        encoder.finish()?.len()
    };

    let zstd = zstd::encode_all(content, 0)?.len();

    Ok((brotli as u64, gzip as u64, zstd as u64))
}

/// Expands `{a,b}` alternation groups the way `@std/path`'s glob does, so
/// patterns such as `{dist,bundle}/**/*.js` (used in the README) work. The
/// `glob` crate does not expand braces itself, so each alternative is expanded
/// into a separate pattern.
///
/// Unbalanced braces, and escapes such as `\{`, are left to the `glob` crate to
/// interpret literally; this differs from `@std/path`, which treats a segment
/// with an unclosed group as a literal.
fn expand_braces(pattern: &str) -> Vec<String> {
    let chars: Vec<char> = pattern.chars().collect();

    // Locate the first `{` and its matching `}`.
    let mut depth = 0usize;
    let mut start = None;
    let mut end = None;
    for (i, &c) in chars.iter().enumerate() {
        match c {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' if depth > 0 => {
                depth -= 1;
                if depth == 0 {
                    end = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let (Some(start), Some(end)) = (start, end) else {
        return vec![pattern.to_string()];
    };

    // Split the group body on commas that are not nested in another group.
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut nested = 0usize;
    for &c in &chars[start + 1..end] {
        match c {
            '{' => {
                nested += 1;
                current.push(c);
            }
            '}' => {
                nested = nested.saturating_sub(1);
                current.push(c);
            }
            ',' if nested == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    parts.push(current);

    let prefix: String = chars[..start].iter().collect();
    let suffix: String = chars[end + 1..].iter().collect();

    let mut expanded = Vec::new();
    for part in parts {
        expanded.extend(expand_braces(&format!("{prefix}{part}{suffix}")));
    }
    expanded
}

#[cfg(test)]
mod tests {
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
}
