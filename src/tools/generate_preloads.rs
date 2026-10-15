//! Port of `tools/generate-preloads/index.ts`.
//!
//! Prints a `<link rel="preload">` tag for every entry in a directory. The
//! original iterates `Deno.readDir` without sorting, so the output order is the
//! OS directory order; this port preserves that behaviour.

use std::fs;

use anyhow::{Context, Result};

use crate::helpers::string_utils::trim_destination;

/// Renders the preload tags for `names` in the order they are given.
pub fn render_preloads(names: &[String], dest: &str) -> Vec<String> {
    let directory = trim_destination(dest);
    names
        .iter()
        .map(|name| {
            format!(
                "<link rel=\"preload\" as=\"font\" crossorigin=\"anonymous\" href=\"~/{directory}/{name}\">"
            )
        })
        .collect()
}

/// Port of `generatePreloads`.
pub fn generate_preloads(path: &str, dest: &str) -> Result<()> {
    let entries = fs::read_dir(path).with_context(|| format!("Failed to read directory {path}"))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("Failed to read entry in {path}"))?;
        names.push(entry.file_name().to_string_lossy().into_owned());
    }

    for line in render_preloads(&names, dest) {
        println!("{line}");
    }

    Ok(())
}

#[cfg(test)]
mod tests;
