# Ogma3 Tools

## Tools

### Bundlesize

Get the size of all bundled Javascript, including after compression with GZip and Brotli

```sh
deno run --allow-read .\index.ts bundlesize <glob>
```

for example

```sh
deno run --allow-read .\index.ts bundlesize "G:\VS Projects\Ogma3\Ogma3\wwwroot\js\{dist,bundle}/**/*.js"
```
