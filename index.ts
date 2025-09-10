import { Command } from "@cliffy/command";
import { bundleSize } from "./tools/bundlesize/index.ts";
import { generateImports } from "./tools/generate-imports/index.ts";
import { generatePaths } from "./tools/generate-paths/generate-paths.ts";
import { generatePreloads } from "./tools/generate-preloads/index.ts";
import stamp from "./stamp.txt" with { type: "text" };

await new Command()
	.command("info")
	.action(() => {
		console.log(`Build time: ${stamp}`);
	})
	.command("bundle-size")
	.arguments("<...paths:string>")
	.description("Calculate total bundle size")
	.action(async (_: unknown, ...paths: string[]) => {
		await bundleSize(paths);
	})
	.command("generate-imports")
	.arguments("<path:string> <dest:string>")
	.description("Generate imports for fonts")
	.action(async (_: unknown, path: string, dest: string) => {
		await generateImports(path, dest);
	})
	.command("generate-preloads")
	.arguments("<path:string> <dest:string>")
	.description("Generate preloads for fonts")
	.action(async (_: unknown, path: string, dest: string) => {
		await generatePreloads(path, dest);
	})
	.command("generate-paths")
	.arguments("<out:string>")
	.option("--path.* [type:string]", "Path to the OpenAPI file", {
		collect: true,
	})
	.option("--verbose", "Verbose output")
	.description("Generate paths")
	.action(async ({ path, verbose }, out: string) => {
		if (!path) {
			throw new Error("No path provided");
		}
		console.log(path);
		await generatePaths(
			out,
			Object.entries(path).flatMap(([k, v]: [string, unknown]) =>
				Array.isArray(v) ? v.map((x: string) => ({ key: k, value: x })) : []
			),
			verbose ?? false,
		);
	})
	.parse(Deno.args);

Deno.exit(0);
