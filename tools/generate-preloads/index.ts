import { trim } from "../../helpers/stringUtils.ts";

export const generatePreloads = async (path: string, dest: string) => {
	const d = trim(trim(dest, "/"), "\\");
	for await (const f of Deno.readDir(path)) {
		console.log(
			`<link rel="preload" as="font" crossorigin="anonymous" href="~/${d}/${f.name}">`,
		);
	}
};
