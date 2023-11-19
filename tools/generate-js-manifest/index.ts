import { expandGlob } from 'https://deno.land/std@0.113.0/fs/expand_glob.ts';
import { WalkEntry } from 'https://deno.land/std@0.143.0/fs/walk.ts';

export const generateManifest = async (glob: string, outfile: string | undefined) => {
	const paths: WalkEntry[] = [];

	for await (const x of expandGlob(glob)) {
		if (x.isFile) {
			paths.push(x);
		}
	}

	const files = paths
		.map((ge) =>
			ge.path
				.split('wwwroot')
				.at(-1)
				?.replaceAll('\\', '/')
		).filter((p) => p && p.length > 0);

	const json = JSON.stringify(files, null, 4);

	if (outfile) {
		await Deno.writeTextFile(outfile, json);
	} else {
		console.log(json);
	}
};
