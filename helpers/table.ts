import { ch } from "./chars.ts";
import { bold } from "@std/fmt/colors";
import { startCase } from "@es-toolkit/es-toolkit";

export class Table {
	private headerArray?: string[];
	private bodyArray: string[][] = [[]];
	private footerArray: string[][] = [[]];

	header(header: string[]) {
		this.headerArray = header;
		return this;
	}

	body(body: string[][]) {
		this.bodyArray = body;
		return this;
	}

	objectBody<T extends object>(body: T[]) {
		this.headerArray = Object.keys(body[0]);
		this.bodyArray = body.map(Object.values);
		return this;
	}

	push(row: string[]) {
		this.bodyArray.push(row);
		return this;
	}

	footer(footer: string[][]) {
		this.footerArray = footer;
		return this;
	}

	toString() {
		return [this.headerArray, ...this.bodyArray, ...this.footerArray].join("\n");
	}

	render() {
		const allRows = [this.headerArray ?? [], ...this.footerArray, ...this.bodyArray];

		const cols = Math.max(...allRows.map((r) => r.length));
		const colWidths = allRows
			.map((r) => r.map((c) => `${c}`.length))
			.reduce((prev, curr) => {
				const out = prev;
				for (let i = 0; i < cols; i++) {
					out[i] = Math.max(prev[i], curr[i] ?? 0);
				}
				return out;
			});

		console.log(this.buildHorizontalBorder(colWidths, "top"));

		if (this.headerArray) {
			console.log(this.buildRow(this.headerArray.map(startCase), colWidths, true));
			console.log(this.buildHorizontalBorder(colWidths, "middle"));
		}

		for (const row of this.bodyArray) {
			console.log(this.buildRow(row, colWidths));
		}

		if (this.footerArray) {
			console.log(this.buildHorizontalBorder(colWidths, "middle"));
			for (const row of this.footerArray) {
				console.log(this.buildRow(row, colWidths));
			}
		}
		console.log(this.buildHorizontalBorder(colWidths, "bottom"));
	}

	private buildRow(row: string[], colWidths: number[], emphasis = false) {
		const rs = [];
		for (const [k, v] of colWidths.entries()) {
			const r = (row[k] ?? "").toString();

			if (emphasis) {
				const em = bold(r);
				rs.push(em.padEnd(v + (em.length - r.length)));
			} else {
				rs.push(r.padEnd(v));
			}
		}
		return `${ch.v} ${rs.join(` ${ch.v} `)} ${ch.v}`;
	}

	private buildHorizontalBorder(
		colWidths: number[],
		position: "top" | "middle" | "bottom",
	) {
		const border = [];
		for (const width of colWidths) {
			border.push(ch.h.repeat(width));
		}
		switch (position) {
			case "top":
				return `${ch.tl}${ch.h}${border.join(`${ch.h}${ch.tm}${ch.h}`)}${ch.h}${ch.tr}`;
			case "middle":
				return `${ch.ml}${ch.h}${border.join(`${ch.h}${ch.mm}${ch.h}`)}${ch.h}${ch.mr}`;
			case "bottom":
				return `${ch.bl}${ch.h}${border.join(`${ch.h}${ch.bm}${ch.h}`)}${ch.h}${ch.br}`;
		}
	}
}
