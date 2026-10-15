# Ogma3 Tools

## Installation

`task install` to install the tool locally (requires https://taskfile.dev/)

## Tools

### Bundle size

Get the size of all bundled Javascript, including after compression with GZip and Brotli

```sh
deno run --allow-read ./index.ts bundle-size <glob>
```

for example

```sh
deno run --allow-read ./index.ts bundle-size "G:\VS Projects\Ogma3\Ogma3\wwwroot\js\{dist,bundle}/**/*.js"
```

### Generate imports

Generates CSS imports for all font files in a given directory

```sh
deno run --allow-read ./index.ts generate-imports <source> <destination>
```

for example

```sh
deno run --allow-read ./index.ts generate-imports "G:\VS Projects\Ogma3\Ogma3\wwwroot\fonts" "fonts"
```

### Generate preloads

Generates `<link>` preloads to speed up the loading of font files

```sh
deno run --allow-read ./index.ts generate-preloads <source> <destination>
```

for example

```sh
deno run --allow-read ./index.ts generate-preloads "G:\VS Projects\Ogma3\Ogma3\wwwroot\fonts" "fonts"
```

### Generate paths

Despite its name, what it really does is it generates a bunch of functions wrapping `fetch()` based on the supplied Swagger URLs.

Clients get outputted to `<destination>` into `<name>.ts` files, one file per supplied URL

```sh
deno run --allow-net --unsafely-ignore-certificate-errors=localhost --allow-read --allow-write ./index.ts generate-paths <destination> --path.<name1>=<url1> --path.<name2>=<url2>
```

for example

```sh
deno run --allow-net --unsafely-ignore-certificate-errors=localhost --allow-read --allow-write ./index.ts generate-paths "out" --path.public="https://localhost:5001/swagger/public/swagger.json" --path.internal="https://localhost:5001/swagger/internal/swagger.json"
```

will result in

```dir
out
|— public.ts
\— internal.ts
```

## Rust rewrite

The tools have been ported to Rust. The original Deno implementation above is
kept for reference; the Rust binary is intended as a drop-in replacement and
preserves the command names, flags, output format and ordering.

### Build and install

```sh
cargo build --release
# -> target/release/ogma-tool
```

`Cargo.lock` is committed, so use `--locked` for reproducible builds.

### Usage

```sh
ogma-tool info
ogma-tool bundle-size "<glob>"...
ogma-tool generate-imports <source> <destination>
ogma-tool generate-preloads <source> <destination>
ogma-tool generate-paths <destination> [-p|--path <name>=<url>]... [-v] [-c] [-k]
```

The original dotted `--path.<name>=<url>` spelling is still accepted, so
existing invocations keep working:

```sh
ogma-tool generate-paths "out" \
  --path.public="https://localhost:5001/openapi/public.json" \
  --path.internal="https://localhost:5001/openapi/private.json"
```

### Checks

```sh
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo clippy --all-targets --all-features -- -D warnings
```

### Differences from the Deno implementation

- TLS certificates: Deno's global `--unsafely-ignore-certificate-errors` is
  replaced by the `generate-paths` flag `-k`/`--insecure`.
- `--path` accepts both the dotted (`--path.name=url`) and the plain
  (`--path name=url`) forms.
- `generate-paths --cache` writes cache files to
  `./.cache/<encodeURIComponent(url)>.json` relative to the current working
  directory. The original stored them next to its own source
  (`tools/generate-paths/.cache`), which has no equivalent for a standalone
  binary.
- `-v` verbose messages approximate Deno's `console.log(value)` object dumps
  rather than being byte-identical.
- zstd output can differ by a few bytes from Deno on large inputs because of
  differences between the linked libzstd builds; gzip and brotli output are
  byte-identical.
- A missing or empty response schema is handled leniently (it produces an empty
  response type) instead of throwing.
- Error text uses Rust/`anyhow` phrasing; command names, flags and exit codes
  match the original.
