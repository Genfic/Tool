# Ogma3 Tools

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
