import { expandGlob, type WalkEntry } from "@std/fs";

export const generateManifest = async (
	glob: string,
	outfile: string | undefined,
) => {
	const paths: WalkEntry[] = [];

	for await (const x of expandGlob(glob)) {
		if (x.isFile) {
			paths.push(x);
		}
	}

	const files = paths
		.map((ge) => ge.path.split("wwwroot").at(-1)?.replaceAll("\\", "/"))
		.filter((p) => p && p.length > 0);

	const json = JSON.stringify(files, null, 4);

	if (outfile) {
		await Deno.writeTextFile(outfile, json);
	} else {
		console.log(json);
	}
};
