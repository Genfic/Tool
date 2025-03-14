import { Command } from "@cliffy/command";
import { bundlesize } from "./tools/bundlesize/index.ts";
import { generateImports } from "./tools/generate-imports/index.ts";
import { generatePaths } from "./tools/generate-paths/generate-paths.ts";
import { generatePreloads } from "./tools/generate-preloads/index.ts";
import { generateManifest } from "./tools/generate-js-manifest/index.ts";

await new Command()
	// Bundlesize
	.command("bundlesize")
	.arguments("<...paths:string>")
	.description("Calculate total bundle size")
	.action(async (_: unknown, ...paths: string[]) => {
		await bundlesize(paths);
	})
	// Generate imports
	.command("generate-imports")
	.arguments("<path:string> <dest:string>")
	.description("Generate imports for fonts")
	.action(async (path: string, dest: string) => {
		await generateImports(path, dest);
	})
	// Generate preloads
	.command("generate-preloads")
	.arguments("<path:string> <dest:string>")
	.description("Generate preloads for fonts")
	.action(async (path: string, dest: string) => {
		await generatePreloads(path, dest);
	})
	// Generate paths
	.command("generate-paths")
	.arguments("<out:string>")
	.option("--path.* [string]", "Path to the OpenAPI file", { collect: true })
	.description("Generate paths")
	.action(
		async ({ path }: { path: { [key: string]: string[] } }, out: string) => {
			await generatePaths(
				out,
				Object.entries(path).flatMap(([k, v]) =>
					v.map((x) => ({ key: k, value: x })),
				),
			);
		},
	)
	// Generate manifest
	.command("generate-js-manifest")
	.arguments("<out:string> [name:string]")
	.description("Generate manifest")
	.action(async (out: string, name: string) => {
		await generateManifest(out, name);
	})
	.parse(Deno.args);

Deno.exit(0);
