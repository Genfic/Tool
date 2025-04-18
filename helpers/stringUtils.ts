export const trim = (source: string, trim: string): string => {
	let start = 0;
	let end = source.length;

	while (start < end && source[start] === trim) ++start;

	while (end > start && source[end - 1] === trim) --end;

	return start > 0 || end < source.length ? source.substring(start, end) : source;
};
