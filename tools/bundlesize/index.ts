import * as path from "@std/path";
import { format as bytes } from "@std/fmt/bytes";
import { expandGlobSync, type WalkEntry } from "@std/fs";
import { Table } from "../../helpers/table.ts";
import type { FileData, PathData } from "./types.ts";
import { brotli, gzip } from "@deno-library/compress";
import { compress as compressZstd } from "@yu7400ki/zstd-wasm";

export const bundleSize = async (paths: string[]) => {
	const p: { [key: string]: PathData[] } = paths
		.flatMap((p) => [...expandGlobSync(p)])
		.map(
			(p: WalkEntry): PathData => ({
				name: p.name,
				dir: path.dirname(p.path),
				path: p.path,
			}),
		)
		.reduce((acc: { [key: string]: PathData[] }, item) => {
			if (acc[item.dir]) {
				acc[item.dir].push(item);
			} else {
				acc[item.dir] = [item];
			}
			return acc;
		}, {});

	const sums = {
		plain: 0,
		brotli: 0,
		gzip: 0,
		zstd: 0,
	};

	for (const path in p) {
		const files = p[path];

		console.log(path);

		const plain: FileData[] = await Promise.all(
			files.map(async (x) => ({
				name: x.name,
				size: (await Deno.stat(x.path)).size,
			})),
		);

		plain.sort((a, b) => b.size - a.size);

		const table = new Table().objectBody(
			plain.map((fd) => ({
				name: fd.name,
				size: bytes(fd.size),
			})),
		);

		const totalSize = plain.map((f) => f.size).reduce((l, c) => l + c);
		const { brotli, gzip, zstd } = await getCompressedSizes(files);

		sums.plain += totalSize;
		sums.brotli += brotli;
		sums.gzip += gzip;
		sums.zstd += zstd;

		table.footer([
			["Total size", bytes(totalSize)],
			["Gzip", `${bytes(gzip)} (${((gzip / totalSize) * 100).toFixed(2)}%)`],
			[
				"Brotli",
				`${bytes(brotli)} (${((brotli / totalSize) * 100).toFixed(2)}%)`,
			],
			["Zstd", `${bytes(zstd)} (${((zstd / totalSize) * 100).toFixed(2)}%)`],
		]);
		table.render();
	}

	console.log(
		`Total JS size: ${bytes(sums.plain)} (${bytes(sums.gzip)} gzipped, ${bytes(sums.brotli)} brotli, ${bytes(sums.zstd)} zstd)`,
	);
};

const getCompressedSizes = async (files: PathData[]) => {
	let f = "";
	for (const we of files) {
		f += await Deno.readTextFile(we.path);
	}
	const content = new TextEncoder().encode(f);
	return {
		brotli: (await brotli.compress(content)).length,
		gzip: gzip(content).length,
		zstd: (await compressZstd(content)).length,
	};
};
