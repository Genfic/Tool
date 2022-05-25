export class Table {
	constructor() {
	}

	private ch = {
		tl: '┌',
		tr: '┐',
		bl: '└',
		br: '┘',
		h: '─',
		v: '│',
		ml: '├',
		mr: '┤',
		tm: '┬',
		bm: '┴',
		mm: '┼',
	};

	#header?: string[];
	#body: string[][] = [[]];
	#footer?: string[];

	header(header: string[]) {
		this.#header = header;
		return this;
	}

	body(body: string[][]) {
		this.#body = body;
		return this;
	}

	objectBody<T>(body: T[]) {
		this.#header = Object.keys(body[0]);
		this.#body = body.map(Object.values);
		return this;
	}

	push(row: string[]) {
		this.#body.push(row);
		return this;
	}

	footer(footer: string[]) {
		this.#footer = footer;
		return this;
	}

	toString() {
		return [
			this.#header,
			...this.#body,
			this.#footer,
		].join('\n');
	}

	render() {
		const allRows = [this.#header ?? [], this.#footer ?? [], ...this.#body];

		const cols = Math.max(...allRows.map((r) => r.length));
		const colWidths = allRows
			.map((r) => r.map((c) => `${c}`.length))
			.reduce((prev, curr) => {
				const out = prev;
				for (let i = 0; i < cols; i++) {
					out[i] = Math.max(prev[i], curr[i]);
				}
				return out;
			});

		console.log(this.buildHorizontalBorder(colWidths, 'top'));

		if (this.#header) {
			console.log(this.buildRow(this.#header, colWidths));
			console.log(this.buildHorizontalBorder(colWidths, 'middle'));
		}

		for (const row of this.#body) {
			console.log(this.buildRow(row, colWidths));
		}

		if (this.#footer) {
			console.log(this.buildHorizontalBorder(colWidths, 'middle'));
			console.log(this.buildRow(this.#footer, colWidths));
		}
		console.log(this.buildHorizontalBorder(colWidths, 'bottom'));
	}

	private buildRow(row: string[], colWidths: number[]) {
		const rs = [];
		for (const [k, v] of colWidths.entries()) {
			rs.push(`${row[k]}`.padEnd(v));
		}
		return `${this.ch.v} ` + rs.join(` ${this.ch.v} `) + ` ${this.ch.v}`;
	}

	private buildHorizontalBorder(colWidths: number[], position: 'top' | 'middle' | 'bottom') {
		const border = [];
		for (const width of colWidths) {
			border.push(this.ch.h.repeat(width));
		}
		switch (position) {
			case 'top':
				return `${this.ch.tl}${this.ch.h}` + border.join(`${this.ch.h}${this.ch.tm}${this.ch.h}`) + `${this.ch.h}${this.ch.tr}`;
			case 'middle':
				return `${this.ch.ml}${this.ch.h}` + border.join(`${this.ch.h}${this.ch.mm}${this.ch.h}`) + `${this.ch.h}${this.ch.mr}`;
			case 'bottom':
				return `${this.ch.bl}${this.ch.h}` + border.join(`${this.ch.h}${this.ch.bm}${this.ch.h}`) + `${this.ch.h}${this.ch.br}`;
		}
	}
}
