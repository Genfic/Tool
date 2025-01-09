import { parseArgs } from "@std/cli";
import { bundlesize } from "./tools/bundlesize/index.ts";
import { generateImports } from "./tools/generate-imports/index.ts";
import { generatePaths } from "./tools/generate-paths/generate-paths.ts";
import { generatePreloads } from "./tools/generate-preloads/index.ts";
import { generateManifest } from "./tools/generate-js-manifest/index.ts";
import { addIcons } from "./tools/icons/icons.ts";

const args = parseArgs(Deno.args, { collect: ["path"] });

const [command, ...params] = args._;

const run = async (): Promise<void> => {
	switch (command) {
		case "bundlesize":
			return await bundlesize(params.map((x) => x.toString()));
		case "generate-imports":
			return await generateImports(params[0].toString(), params[1].toString());
		case "generate-preloads":
			return await generatePreloads(params[0].toString(), params[1].toString());
		case "generate-paths":
			return await generatePaths(
				params[0].toString(),
				args.path as unknown as { [key: string]: string },
			);
		case "generate-js-manifest":
			return await generateManifest(
				params[0].toString(),
				params.at(1)?.toString(),
			);
		case "icons":
			return await addIcons(params[0].toString());
		case "demo":
			return await demo({ command, params, args });
		default:
			console.log(`[${command}] is not a correct option`);
	}
};
await run();

async function demo(obj: object) {
	await Promise.resolve();
	console.info(JSON.stringify(obj));
}
