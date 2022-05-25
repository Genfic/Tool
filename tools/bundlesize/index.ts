import { expandGlobSync } from 'https://deno.land/std@0.113.0/fs/mod.ts';
import { Table } from '../../helpers/table.ts';
import { FileData } from './types.ts';

export const bundlesize = (path: string) => {
	const files: FileData[] = [...expandGlobSync(path)]
		.filter((x) => !x.path.includes('/admin/'))
		.map((x) => ({ name: x.name, size: Deno.statSync(x.path).size }));

	files.sort((a, b) => b.size - a.size);

	const table = new Table()
		.objectBody(files);

	const totalSize = files.map((f) => f.size).reduce((l, c) => l + c);

	table.footer(['total size', totalSize.toString()]);
	table.render();
};
