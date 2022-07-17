# Ogma3 Tools

## Installation

`vr install` to install the tool locally

## Tools

### Bundlesize

Get the size of all bundled Javascript, including after compression with GZip and Brotli

```sh
deno run --allow-read ./index.ts bundlesize <glob>
```

for example

```sh
deno run --allow-read ./index.ts bundlesize "G:\VS Projects\Ogma3\Ogma3\wwwroot\js\{dist,bundle}/**/*.js"
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

### Genrate preloads

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
