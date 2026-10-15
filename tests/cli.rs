//! End-to-end tests that exercise the compiled `ogma-tool` binary against
//! temporary fixtures. They cover the observable CLI contract: the commands,
//! their output format, ordering and error handling.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};
use std::thread;

use tempfile::TempDir;

/// Builds a command for the compiled binary with colour disabled so the table
/// borders and headers are stable.
fn bin() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ogma-tool"));
    command.env("NO_COLOR", "1");
    command
}

fn run(args: &[&str]) -> Output {
    bin().args(args).output().expect("failed to run ogma-tool")
}

fn stdout_of(output: &Output) -> String {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).expect("stdout was not UTF-8")
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

#[test]
fn info_prints_the_build_stamp() {
    let output = run(&["info"]);
    let stdout = stdout_of(&output);
    // The embedded stamp mirrors `stamp.txt`, including its trailing CRLF, and
    // `println!` adds the final newline (like `console.log`).
    assert_eq!(stdout.trim_end(), "Build time: 2026-06-20 21:21:34");
}

#[test]
fn generate_imports_prints_sorted_css_source_list() {
    let dir = TempDir::new().unwrap();
    let fonts = dir.path().join("fonts");
    write(&fonts.join("roboto-latin-400.woff2"), "a");
    write(&fonts.join("roboto-latin-400.woff"), "bb");
    write(&fonts.join("LICENSE"), "ccc");

    let output = run(&["generate-imports", fonts.to_str().unwrap(), "/fonts/"]);

    assert_eq!(
        stdout_of(&output),
        "url('/fonts/roboto-latin-400.woff2') format('woff2'),\n\
         url('/fonts/roboto-latin-400.woff') format('woff'),\n\
         url('/fonts/LICENSE') format('undefined'),\n"
    );
}

#[test]
fn generate_preloads_prints_a_link_for_each_file() {
    let dir = TempDir::new().unwrap();
    let fonts = dir.path().join("fonts");
    write(&fonts.join("a.woff2"), "a");
    write(&fonts.join("b.woff"), "b");

    let output = run(&["generate-preloads", fonts.to_str().unwrap(), "\\fonts\\"]);
    let stdout = stdout_of(&output);
    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.sort_unstable();

    assert_eq!(
        lines,
        [
            "<link rel=\"preload\" as=\"font\" crossorigin=\"anonymous\" href=\"~/fonts/a.woff2\">",
            "<link rel=\"preload\" as=\"font\" crossorigin=\"anonymous\" href=\"~/fonts/b.woff\">",
        ]
    );
}

#[test]
fn generate_imports_errors_on_a_missing_directory() {
    let output = run(&["generate-imports", "/does/not/exist/at/all", "fonts"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Failed to read directory"),
        "unexpected stderr: {stderr}"
    );
}

#[test]
fn bundle_size_reports_plain_and_compressed_totals() {
    let dir = TempDir::new().unwrap();
    write(&dir.path().join("dist/a.js"), "function a(){return 1}\n");
    write(
        &dir.path().join("bundle/b.js"),
        "function b(){return a()+2}\nfunction c(){return b()}\n",
    );
    write(&dir.path().join("bundle/d.js"), "export const d = 42;\n");

    // Uses the `{dist,bundle}` brace expansion from the README.
    let pattern = format!("{}/{{dist,bundle}}/*.js", dir.path().display());
    let output = run(&["bundle-size", &pattern]);
    let stdout = stdout_of(&output);

    assert!(stdout.contains(&format!("{}", dir.path().join("dist").display())));
    assert!(stdout.contains(&format!("{}", dir.path().join("bundle").display())));
    // Plain sizes are deterministic; the compressed sizes depend on the
    // compression backends.
    assert!(stdout.contains("│ a.js       │ 23 B "));
    assert!(stdout.contains("│ b.js       │ 52 B "));
    assert!(stdout.contains("│ d.js       │ 21 B "));
    assert!(stdout.contains("│ Total size │ 23 B "));
    assert!(stdout.contains("│ Total size │ 73 B "));

    // `Total JS size: 96 B (... gzipped, ... brotli, ... zstd)`.
    let last = stdout.lines().last().unwrap();
    let rest = last
        .strip_prefix("Total JS size: 96 B (")
        .expect("missing total line");
    let rest = rest.strip_suffix(")").expect("total line not closed");
    let parts: Vec<&str> = rest.split(", ").collect();
    assert_eq!(parts.len(), 3);
    assert!(parts[0].ends_with(" B gzipped"), "{}", parts[0]);
    assert!(parts[1].ends_with(" B brotli"), "{}", parts[1]);
    assert!(parts[2].ends_with(" B zstd"), "{}", parts[2]);
}

#[test]
fn bundle_size_with_no_matches_reports_zero() {
    let dir = TempDir::new().unwrap();
    let pattern = format!("{}/*.js", dir.path().display());
    let output = run(&["bundle-size", &pattern]);
    assert_eq!(
        stdout_of(&output),
        "Total JS size: 0 B (0 B gzipped, 0 B brotli, 0 B zstd)\n"
    );
}

#[test]
fn generate_paths_requires_at_least_one_path() {
    let output = run(&["generate-paths", "out"]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("No path provided"),
        "unexpected stderr"
    );
}

#[test]
fn generate_paths_rejects_a_malformed_path_option() {
    let output = run(&["generate-paths", "out", "--path", "no-equals"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid --path value"),
        "unexpected stderr: {stderr}"
    );
}

/// Serves `body` once over HTTP on a random local port and returns the port.
fn serve_once(body: &'static str) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        // Read the request headers (we ignore the body; there is none).
        let mut buffer = [0u8; 1024];
        let _ = stream.read(&mut buffer);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        let _ = stream.write_all(response.as_bytes());
        let _ = stream.flush();
    });
    port
}

const SPEC: &str = r##"{
  "openapi": "3.0.0",
  "paths": {
    "/items/{item-id}": {
      "get": {
        "operationId": "getItem",
        "parameters": [
          {"name": "item-id", "in": "path", "required": true, "schema": {"type": "integer"}},
          {"name": "search_term", "in": "query", "required": false, "schema": {"type": "string"}}
        ],
        "responses": {
          "200": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Item"}}}}
        }
      }
    }
  },
  "components": {
    "schemas": {
      "Item": {
        "type": "object",
        "properties": {"id": {"type": "integer"}, "name": {"type": "string"}}
      }
    }
  }
}"##;

#[test]
fn generate_paths_fetches_a_spec_and_writes_the_client_files() {
    let port = serve_once(SPEC);
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("out");
    let url = format!("http://127.0.0.1:{port}/spec.json");

    let output = run(&[
        "generate-paths",
        out.to_str().unwrap(),
        "--path",
        &format!("test={url}"),
    ]);
    let stdout = stdout_of(&output);

    // The collected path map is echoed like `console.log(path)`.
    assert_eq!(
        stdout.lines().next().unwrap(),
        format!("{{ test: [ \"{url}\" ] }}")
    );
    assert!(stdout.contains("Generating paths for: test"));

    // The runtime is copied verbatim.
    assert_eq!(
        fs::read_to_string(out.join("typed-fetch.ts")).unwrap(),
        ogma_tools::tools::generate_paths::TYPED_FETCH
    );

    // The rendered files must match the pure renderer for the same document.
    let value: serde_json::Value = serde_json::from_str(SPEC).unwrap();
    let generated = ogma_tools::tools::generate_paths::generate_value(value, false).unwrap();
    let expected_paths = ogma_tools::tools::generate_paths::render::render_paths_file(
        "test",
        &generated.type_imports,
        &generated.paths,
    );
    assert_eq!(
        fs::read_to_string(out.join("paths-test.ts")).unwrap(),
        expected_paths
    );
    assert_eq!(
        fs::read_to_string(out.join("types-test.ts")).unwrap(),
        generated.types.join("\n\n")
    );

    // Sanity-check the generated TypeScript.
    let paths = fs::read_to_string(out.join("paths-test.ts")).unwrap();
    assert!(paths.contains("getItem"), "paths file: {paths}");
    assert!(paths.contains("${itemId}"), "paths file: {paths}");
    let types = fs::read_to_string(out.join("types-test.ts")).unwrap();
    assert!(types.contains("export type Item = "), "types file: {types}");
}
