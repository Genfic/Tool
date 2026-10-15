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
