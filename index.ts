import { Command } from "@cliffy/command";
import { bundlesize } from "./tools/bundlesize/index.ts";
import { generateImports } from "./tools/generate-imports/index.ts";
import { generatePaths } from "./tools/generate-paths/generate-paths.ts";
import { generatePreloads } from "./tools/generate-preloads/index.ts";

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
	.action(async (_: unknown, path: string, dest: string) => {
		await generateImports(path, dest);
	})
	// Generate preloads
	.command("generate-preloads")
	.arguments("<path:string> <dest:string>")
	.description("Generate preloads for fonts")
	.action(async (_: unknown, path: string, dest: string) => {
		await generatePreloads(path, dest);
	})
	// Generate paths
	.command("generate-paths")
	.arguments("<out:string>")
	.option("--path.* [type:string]", "Path to the OpenAPI file", {
		collect: true,
	})
	.description("Generate paths")
	.action(async ({ path }, out: string) => {
		if (!path) {
			throw new Error("No path provided");
		}
		console.log(path);
		await generatePaths(
			out,
			Object.entries(path).flatMap(([k, v]: [string, unknown]) =>
				Array.isArray(v) ? v.map((x: string) => ({ key: k, value: x })) : [],
			),
		);
	})
	.parse(Deno.args);

Deno.exit(0);
