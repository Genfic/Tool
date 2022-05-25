import { expandGlobSync, WalkEntry } from 'https://deno.land/std@0.113.0/fs/mod.ts';
import { Table } from '../../helpers/table.ts';
import { FileData } from './types.ts';
import { compress as compressBrotli } from 'https://deno.land/x/brotli/mod.ts';
import { gzip as compressGzip } from 'https://deno.land/x/compress@v0.4.4/mod.ts';

export const bundlesize = async (path: string) => {
	const files: WalkEntry[] = [...expandGlobSync(path)]
		.filter((x) => !x.path.includes('/admin/'));

	const plain: FileData[] = files
		.map((x) => ({ name: x.name, size: Deno.statSync(x.path).size }));

	plain.sort((a, b) => b.size - a.size);

	const table = new Table()
		.objectBody(plain);

	const totalSize = plain.map((f) => f.size).reduce((l, c) => l + c);

	const { brotli, gzip } = await getCompressedSizes(files);
	table.footer([
		['Total size', totalSize.toString()],
		['Gzip', `${gzip} (${(gzip / totalSize * 100).toFixed(2)}%)`],
		['Brotli', `${brotli} (${(brotli / totalSize * 100).toFixed(2)}%)`],
	]);
	table.render();
};

const getCompressedSizes = async (files: WalkEntry[]) => {
	let f = '';
	for (const we of files) {
		f += await Deno.readTextFile(we.path);
	}
	const content = new TextEncoder().encode(f);
	return {
		brotli: compressBrotli(content).length,
		gzip: compressGzip(content).length,
	};
};
