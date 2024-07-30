import { trim } from "../../helpers/stringUtils.ts";

const priorities = new Map<string, number>([
	["woff2", 1],
	["woff", 2],
	["otf", 3],
	["ttf", 4],
]);

export const generateImports = async (source: string, dest: string) => {
	const d = trim(trim(dest, "/"), "\\");
	const tags: { tag: string; priority: number }[] = [];

	// const path = new URL(source, import.meta.url);
	for await (const f of Deno.readDir(source)) {
		const name = f.name.split(".")[1];
		tags.push({ tag: f.name, priority: priorities.get(name) ?? 0 });
	}

	tags.sort((a, b) => a.priority - b.priority);
	tags.sort((a, b) => (a.tag > b.tag ? 1 : b.tag > a.tag ? -1 : 0)).reverse();

	for (const t of tags) {
		console.log(`url('/${d}/${t.tag}') format('${t.tag.split(".")[1]}'),`);
	}
};
