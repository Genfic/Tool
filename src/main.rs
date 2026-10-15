//! Command line entry point for the Rust rewrite of the Ogma3 build tools.
//!
//! The subcommands and their arguments mirror `index.ts` (a
//! [`@cliffy/command`](https://jsr.io/@cliffy/command) CLI):
//!
//! * `info`
//! * `bundle-size <...paths>`
//! * `generate-imports <path> <dest>`
//! * `generate-preloads <path> <dest>`
//! * `generate-paths <out> [-p|--path NAME=URL]... [-v] [-c] [-k]`
//!
//! See `README.md` for the differences from the Deno implementation.

use std::ffi::OsString;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use indexmap::IndexMap;

use ogma_tools::tools::bundlesize::bundle_size;
use ogma_tools::tools::generate_imports::generate_imports;
use ogma_tools::tools::generate_paths::generate_paths;
use ogma_tools::tools::generate_preloads::generate_preloads;

/// Build stamp embedded at compile time, equivalent to reading `stamp.txt` with
/// `import stamp from "./stamp.txt" with { type: "text" }`. The checked-in file
/// contains a trailing CRLF.
const STAMP: &str = include_str!("../stamp.txt");

#[derive(Parser)]
#[command(name = "ogma-tool", version, about = "Ogma3 build tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print build info
    Info,
    /// Calculate total bundle size
    #[command(name = "bundle-size")]
    BundleSize {
        /// Glob patterns to include
        #[arg(required = true, value_name = "PATH")]
        paths: Vec<String>,
    },
    /// Generate imports for fonts
    #[command(name = "generate-imports")]
    GenerateImports {
        /// Directory containing the font files
        path: String,
        /// Destination prefix used in the generated URLs
        dest: String,
    },
    /// Generate preloads for fonts
    #[command(name = "generate-preloads")]
    GeneratePreloads {
        /// Directory containing the font files
        path: String,
        /// Destination prefix used in the generated URLs
        dest: String,
    },
    /// Generate paths
    #[command(name = "generate-paths")]
    GeneratePaths {
        /// Output directory
        out: String,
        /// Path to the OpenAPI file, as NAME=URL (repeatable)
        #[arg(short = 'p', long = "path", value_name = "NAME=URL")]
        path: Vec<String>,
        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
        /// Cache the OpenAPI spec to a file
        #[arg(short = 'c', long)]
        cache: bool,
        /// Do not verify TLS certificates (replaces Deno's
        /// `--unsafely-ignore-certificate-errors`)
        #[arg(short = 'k', long)]
        insecure: bool,
    },
}

fn main() {
    // Rewrite clap-incompatible `--path.NAME=URL` / `--path.NAME URL` forms into
    // the `--path NAME=URL` form that clap understands.
    let args = normalize_path_args(std::env::args_os());
    let cli = Cli::parse_from(args);

    if let Err(error) = run(cli) {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Info => {
            println!("Build time: {STAMP}");
            Ok(())
        }
        Command::BundleSize { paths } => bundle_size(&paths),
        Command::GenerateImports { path, dest } => generate_imports(&path, &dest),
        Command::GeneratePreloads { path, dest } => generate_preloads(&path, &dest),
        Command::GeneratePaths {
            out,
            path,
            verbose,
            cache,
            insecure,
        } => {
            let grouped = group_paths(&path)?;
            if grouped.is_empty() {
                bail!("No path provided");
            }
            // The original logs the collected `--path` object here.
            println!("{}", format_path_map(&grouped));

            let specs = flatten_paths(&grouped);
            generate_paths(&out, &specs, verbose, cache, insecure)
        }
    }
}

/// Collects `NAME=URL` arguments into an ordered map, grouping repeated names
/// together (matching `Object.entries` over cliffy's collected option object).
fn group_paths(args: &[String]) -> Result<IndexMap<String, Vec<String>>> {
    let mut grouped: IndexMap<String, Vec<String>> = IndexMap::new();
    for arg in args {
        let Some((name, value)) = arg.split_once('=') else {
            bail!("Invalid --path value `{arg}`: expected NAME=URL");
        };
        if name.is_empty() {
            bail!("Invalid --path value `{arg}`: the name must not be empty");
        }
        grouped
            .entry(name.to_string())
            .or_default()
            .push(value.to_string());
    }
    Ok(grouped)
}

/// `Object.entries(path).flatMap(...)` — one `(name, url)` pair per value, in
/// first-seen key order.
fn flatten_paths(grouped: &IndexMap<String, Vec<String>>) -> Vec<(String, String)> {
    grouped
        .iter()
        .flat_map(|(name, values)| {
            values
                .iter()
                .map(|value| (name.clone(), value.clone()))
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Best-effort reproduction of Deno's `console.log(path)` for the collected
/// option object, e.g. `{ public: [ "https://…/public.json" ] }`.
fn format_path_map(grouped: &IndexMap<String, Vec<String>>) -> String {
    let entries = grouped
        .iter()
        .map(|(name, values)| {
            let values = values
                .iter()
                .map(|value| format!("\"{}\"", escape_js_string(value)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}: [ {values} ]")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{ {entries} }}")
}

fn escape_js_string(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Rewrites `--path.NAME=URL`, `--path.NAME URL`, `-p.NAME=URL` and
/// `-p.NAME URL` into the `--path NAME=URL` form clap accepts. All other
/// arguments are passed through unchanged.
fn normalize_path_args<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut normalized = Vec::new();
    let mut iter = args.into_iter().peekable();

    while let Some(arg) = iter.next() {
        let text = arg.to_string_lossy().into_owned();
        let dotted = text
            .strip_prefix("--path.")
            .or_else(|| text.strip_prefix("-p."))
            .map(str::to_owned);

        match dotted {
            Some(rest) if !rest.is_empty() => {
                normalized.push(OsString::from("--path"));
                if rest.contains('=') {
                    normalized.push(OsString::from(rest));
                } else if let Some(value) = iter.next() {
                    normalized.push(OsString::from(format!(
                        "{rest}={}",
                        value.to_string_lossy()
                    )));
                } else {
                    normalized.push(OsString::from(rest));
                }
            }
            _ => normalized.push(arg),
        }
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::{flatten_paths, format_path_map, group_paths, normalize_path_args};
    use clap::Parser;
    use std::ffi::OsString;

    fn cli(args: &[&str]) -> super::Cli {
        super::Cli::parse_from(args)
    }

    #[test]
    fn info_parses() {
        assert!(matches!(
            cli(&["ogma-tool", "info"]).command,
            super::Command::Info
        ));
    }

    #[test]
    fn generate_paths_parses_short_flags() {
        let parsed = cli(&[
            "ogma-tool",
            "generate-paths",
            "out",
            "-p",
            "public=https://example.test/public.json",
            "-v",
            "-c",
            "-k",
        ]);
        match parsed.command {
            super::Command::GeneratePaths {
                out,
                path,
                verbose,
                cache,
                insecure,
            } => {
                assert_eq!(out, "out");
                assert_eq!(path, ["public=https://example.test/public.json"]);
                assert!(verbose && cache && insecure);
            }
            _ => panic!("expected generate-paths"),
        }
    }

    #[test]
    fn normalize_rewrites_dotted_path_options() {
        let input = [
            "ogma-tool",
            "generate-paths",
            "out",
            "--path.public=https://a",
            "--path.private",
            "https://b",
            "-p.internal=https://c",
        ]
        .into_iter()
        .map(OsString::from)
        .collect::<Vec<_>>();

        let normalized: Vec<String> = normalize_path_args(input)
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();

        assert_eq!(
            normalized,
            [
                "ogma-tool",
                "generate-paths",
                "out",
                "--path",
                "public=https://a",
                "--path",
                "private=https://b",
                "--path",
                "internal=https://c",
            ]
        );
    }

    #[test]
    fn groups_repeated_names_and_flattens_in_first_seen_order() {
        let args = vec!["a=1".to_string(), "b=2".to_string(), "a=3".to_string()];
        let grouped = group_paths(&args).unwrap();
        assert_eq!(grouped.keys().cloned().collect::<Vec<_>>(), ["a", "b"]);
        assert_eq!(
            flatten_paths(&grouped),
            vec![
                ("a".to_string(), "1".to_string()),
                ("a".to_string(), "3".to_string()),
                ("b".to_string(), "2".to_string()),
            ]
        );
    }

    #[test]
    fn formats_the_path_map_like_deno_console_log() {
        let grouped = group_paths(&[
            "public=https://a".to_string(),
            "private=https://b".to_string(),
        ])
        .unwrap();
        assert_eq!(
            format_path_map(&grouped),
            "{ public: [ \"https://a\" ], private: [ \"https://b\" ] }"
        );
    }

    #[test]
    fn rejects_malformed_path_arguments() {
        assert!(group_paths(&["no-equals".to_string()]).is_err());
        assert!(group_paths(&["=value".to_string()]).is_err());
    }
}
