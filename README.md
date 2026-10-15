# Ogma3 Tools

A small Rust CLI of build-time helpers for the Ogma3 project:

| Command            | Purpose                                                        |
| ------------------ | -------------------------------------------------------------- |
| `info`             | Print build info (version, build timestamp, git revision).     |
| `bundle-size`      | Total size of bundled JS, raw and gzip/brotli/zstd compressed. |
| `generate-imports` | Generate CSS `@import` rules for the font files in a folder.   |
| `generate-preloads`| Generate `<link rel="preload">` tags for the font files.       |
| `generate-paths`   | Generate typed `fetch()` wrappers from OpenAPI documents.      |

The binary is named `ogma-tool`. It is a drop-in replacement for the original
Deno implementation and preserves command names, flags, output format and
ordering (see [Differences from the Deno implementation](#differences-from-the-deno-implementation)).

## Installation

### Prebuilt binaries

Every `v*` tag builds and uploads binaries for Linux (x86_64, aarch64),
macOS (x86_64, arm64) and Windows (x86_64) to the GitHub release. Install the
right one for your platform with [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall):

```sh
cargo binstall ogma-tools
# or, straight from the repository without a crates.io publish:
cargo binstall --git https://github.com/Genfic/Tool ogma-tools
```

Release assets are bare binaries named `ogma-tool-<target-triple>[.exe]`, which
is what `cargo-binstall` expects (configured via `[package.metadata.binstall]`
in `Cargo.toml`). You can also download an asset manually and put it on your
`PATH`.

### From source

```sh
cargo build --release
# -> target/release/ogma-tool
```

`Cargo.lock` is committed, so use `--locked` for reproducible builds:

```sh
cargo install --locked --path .
```

### With Task

If you have [Task](https://taskfile.dev/) installed:

```sh
task install   # builds and `cargo install`s the tool
```

## Usage

```sh
ogma-tool info

ogma-tool bundle-size <glob>...

ogma-tool generate-imports  <source> <destination>
ogma-tool generate-preloads <source> <destination>

ogma-tool generate-paths <destination> [-p|--path <name>=<url>]... [-v] [-c] [-k]
```

### info

Prints the embedded build metadata:

```sh
ogma-tool info
# ogma-tool 0.1.0 (2026-10-15 22:08:03, git abc1234)
```

The timestamp and revision are embedded at compile time by `build.rs`; there is
no checked-in `stamp.txt` to keep in sync. `SOURCE_DATE_EPOCH` is honoured for
reproducible builds, and the git revision is simply omitted when git is not
available.

### bundle-size

Get the size of all bundled JavaScript, including after compression with gzip
and brotli.

```sh
ogma-tool bundle-size "G:\VS Projects\Ogma3\Ogma3\wwwroot\js\{dist,bundle}/**/*.js"
```

### generate-imports

Generate CSS `@import` rules for every font file in a directory.

```sh
ogma-tool generate-imports "G:\VS Projects\Ogma3\Ogma3\wwwroot\fonts" "fonts"
```

### generate-preloads

Generate `<link rel="preload">` tags to speed up font loading.

```sh
ogma-tool generate-preloads "G:\VS Projects\Ogma3\Ogma3\wwwroot\fonts" "fonts"
```

### generate-paths

Despite the name, this generates a set of functions that wrap `fetch()` based on
the supplied OpenAPI documents. For each `--path <name>=<url>` it writes
`paths-<name>.ts` and `types-<name>.ts` into `<destination>`, plus a shared
`typed-fetch.ts` runtime.

```sh
ogma-tool generate-paths <destination> --path.<name1>=<url1> --path.<name2>=<url2>
```

For example:

```sh
ogma-tool generate-paths "out" \
  --path.public="https://localhost:5001/openapi/public.json" \
  --path.internal="https://localhost:5001/openapi/internal.json"
```

produces:

```dir
out
├─ paths-public.ts
├─ paths-internal.ts
├─ types-public.ts
├─ types-internal.ts
└─ typed-fetch.ts
```

Options:

- `-p, --path NAME=URL` — an OpenAPI document; repeatable. Both the dotted
  (`--path.name=url`) and the plain (`--path name=url`) spellings are accepted.
- `-v, --verbose` — verbose progress output.
- `-c, --cache` — cache fetched specs to `./.cache/<encodeURIComponent(url)>.json`.
- `-k, --insecure` — do not verify TLS certificates (replaces Deno's
  `--unsafely-ignore-certificate-errors`; handy for a local dev server with a
  self-signed certificate).

## Development

Run the same checks that CI runs:

```sh
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo clippy --all-targets --all-features -- -D warnings
```

With Task: `task fmt`, `task check`, `task test`, `task clippy`, `task fmt-fix`.

CI (`.github/workflows/ci.yml`) runs fmt/clippy on Linux and the test suite on
Linux and Windows. `.github/workflows/release.yml` builds and publishes the
platform binaries when a `v*` tag is pushed.

## Differences from the Deno implementation

- TLS certificates: Deno's global `--unsafely-ignore-certificate-errors` is
  replaced by the `generate-paths` flag `-k`/`--insecure`.
- `--path` accepts both the dotted (`--path.name=url`) and the plain
  (`--path name=url`) forms.
- `generate-paths --cache` writes cache files to
  `./.cache/<encodeURIComponent(url)>.json` relative to the current working
  directory. The original stored them next to its own source
  (`tools/generate-paths/.cache`), which has no equivalent for a standalone
  binary.
- Build info is embedded at compile time by `build.rs` instead of being read
  from a checked-in `stamp.txt`.
- `-v` verbose messages approximate Deno's `console.log(value)` object dumps
  rather than being byte-identical.
- zstd output can differ by a few bytes from Deno on large inputs because of
  differences between the linked libzstd builds; gzip and brotli output are
  byte-identical.
- A missing or empty response schema is handled leniently (it produces an empty
  response type) instead of throwing.
- Error text uses Rust/`anyhow` phrasing; command names, flags and exit codes
  match the original.

## License

MIT
