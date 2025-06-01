import { ensureDir } from "@std/fs";
import { uniq } from "@es-toolkit/es-toolkit";
import { Eta } from "@eta-dev/eta";
import * as path from "@std/path";
import { match, P } from "@dewars/pattern";
import { camelCase } from "@es-toolkit/es-toolkit";

import type { Parameter, Response as Res, Route, SwaggerResponse, Type } from "./types.ts";

const __dirname = path.dirname(path.fromFileUrl(import.meta.url));
const eta = new Eta({
	views: path.join(__dirname, "templates"),
	autoEscape: false,
	autoFilter: true,
	cache: true,
	filterFunction: (value): string => {
		if (value === undefined || value === null) return "";
		return value.toString();
	},
});

let VERBOSE = false;
let CURRENT_FUNCTION = "";

const replaceType = (type: string): string => {
	return (
		{
			integer: "number",
			decimal: "number",
			double: "number",
			float: "number",
			bool: "boolean",
			string: "string",
			date: "Date",
		}[type] ?? "unknown"
	);
};

const getTypeActual = (type: Type): { type: string; nullable?: boolean } => {
	VERBOSE && console.log("Getting type actual for", type);

	const t: { type: string; nullable?: boolean } = {
		type: "unknown",
		nullable: type.nullable,
	};

	if (type.format === "date-time") {
		t.type = "date";
	} else if (typeof type.type === "string") {
		if (type.type === "object" && type.additionalProperties) {
			const { type: at, nullable: an } = getTypeActual(
				type.additionalProperties,
			);
			t.type = at;
			t.nullable = an;
		} else {
			t.type = type.type;
		}
	} else if (type.format?.startsWith("int") || type.type.includes("integer")) {
		t.type = "integer";
	} else if (type.type.includes("string")) {
		t.type = "string";
	}

	if (type.type.includes("null")) {
		t.nullable = true;
	}
	return t;
};

const buildParams = (parameters: Parameter[]): string[] => {
	VERBOSE && console.log(`Building params for ${CURRENT_FUNCTION}`);

	const params = [];
	for (const param of parameters) {
		const { type } = getTypeActual(param.schema);

		const nullable = !param.required;

		VERBOSE && console.log("Param", param, type, nullable);

		if (param.in !== "path" && param.in !== "query") continue;
		const p = `${camelCase(param.name)}: ${replaceType(type)}${nullable ? " | null" : ""}`;
		params.push(p);
	}
	return params;
};

const buildQuery = (parameters: Parameter[]): string => {
	const params = [];
	for (const param of parameters) {
		if (param.in !== "query") continue;
		let p = camelCase(param.name);
		p += "=${";
		p += camelCase(param.name);
		p += "}";
		params.push(p);
	}
	return params.length > 0 ? `?${params.join("&")}` : "";
};

const parseType = (
	schema: Type,
): [typeString: string | undefined, skipImport: boolean, description?: string] => {
	VERBOSE && console.log("Parsing type", schema);

	const typeMappings: { [key: string]: string } = {
		integer: "number",
		date: "Date",
		undefined: "object",
		array: "object[]",
	};

	const extract = (ref: string | undefined) => ref?.split("/").at(-1);

	let nullable = schema.nullable ?? false;

	const ret = match<
		Type,
		[typeString: string | undefined, skipImport: boolean]
	>(schema)
		.with({ $ref: P.nonNullable }, (s) => [extract(s.$ref), false])
		.with({ format: "binary" }, () => ["Blob", true])
		.with({ items: P.nonNullable }, (s) => {
			const [t, skip] = parseType(s.items);
			return [`${t}[]`, skip];
		})
		.with({ enum: P.nonNullable }, (s) => {
			return [s.enum.map((e) => `"${e}"`).join(" | "), true];
		})
		.with({ oneOf: P.nonNullable }, (s) => {
			const subtypes = [];
			for (const t of s.oneOf) {
				const [parsed, _] = parseType(t);
				if (!parsed) {
					continue;
				}
				subtypes.push(parsed);
			}
			return [subtypes.join(" | "), true];
		})
		.with({ properties: P.nonNullable }, (s) => {
			const props: { name: string; type: string; description?: string }[] = [];
			for (const [key, value] of Object.entries(s.properties)) {
				const [parsed, _] = parseType(value);
				if (!parsed) {
					continue;
				}
				props.push({ name: key, type: parsed, description: value.description });
			}
			return [eta.render("type", { props }), true];
		})
		.with({ type: P.nonNullable }, (s) => {
			const { type, nullable: nl } = getTypeActual(s);
			nullable = nl ?? false;
			return [typeMappings[type] ?? type, true];
		})
		.otherwise(() => [undefined, true]);

	if (nullable) {
		ret[0] += " | null";
	}

	return ret;
};

const buildResponseType = (response: Res): [string | undefined, boolean, string?] => {
	if (!response.content) {
		return [undefined, true];
	}

	const schema = match(response.content)
		.with(
			{ "application/json": P.nonNullable },
			(s) => s["application/json"].schema,
		)
		.with(
			{ "application/octet-stream": P.nonNullable },
			(s) => s["application/octet-stream"].schema,
		)
		.exhaustive();

	return parseType(schema);
};

const buildFunction = (
	path: string,
	method: string,
	meta: Route,
	schemas: { [key: string]: Type },
): {
	func: string;
	type: string | null;
	responseTypes: string[];
} => {
	CURRENT_FUNCTION = meta.operationId;
	VERBOSE && console.log("Building function for", meta.operationId, meta);

	const id = meta.operationId;
	const params = meta.parameters ? buildParams(meta.parameters) : null;
	const url = path
		.replaceAll(/\{(.+)}/gi, (c) => `{${camelCase(c)}}`)
		.replaceAll("{", "${");
	const query = meta.parameters ? buildQuery(meta.parameters) : "";

	// If the path or the query contains `{}`s, that means the resulting string has to use an interpolated string
	const q = [path, query].some((s) => ["{", "}"].every((c) => s.includes(c))) ? "`" : '"';

	const bodyRef = meta.requestBody &&
		"application/json" in meta.requestBody.content &&
		meta.requestBody.content["application/json"].schema;

	const [bodyType, _] = bodyRef ? parseType(bodyRef) : [undefined, false];

	// Check if body type is an empty type
	const isNotEmpty = bodyType && schemas[bodyType]?.properties !== undefined;

	const responseTypes: [string | undefined, boolean, string?][] = [];
	for (const [_, response] of Object.entries(meta.responses)) {
		const t = buildResponseType(response);
		if (t) {
			responseTypes.push(t);
		}
	}

	const responseType: string = responseTypes.every(([t, _]) => t === undefined)
		? "void"
		: uniq(responseTypes.map(([t, _]) => t).filter((t) => !!t)).join("|");

	const func = eta.render("./function", {
		id,
		params,
		responseType,
		q,
		url,
		query,
		method,
		bodyType,
		isNotEmpty,
	});

	return {
		func,
		type: bodyType ?? null,
		responseTypes: responseTypes
			.filter(([_, p]) => !p)
			.map(([t, _]) => t?.replace("[]", ""))
			.filter((x) => !!x) as string[],
	};
};

const buildType = (name: string, component: Type): string | null => {
	const [t, _] = parseType(component);
	if (t) {
		return `export type ${name} = ${t};`;
	}

	return `export type ${name} = Record<string, never>;`;
};

const generate = async (
	path: string,
): Promise<{
	paths: string[];
	types: string[];
	typeImports: string[];
}> => {
	const res = await fetch(path);
	const data: SwaggerResponse = await res.json();

	const paths = [];
	let typeImports: string[] = [];

	for (const [k, v] of Object.entries(data.paths)) {
		for (const [pathKey, pathVal] of Object.entries(v)) {
			const { func, type, responseTypes } = buildFunction(
				k,
				pathKey,
				pathVal as Route,
				data.components.schemas,
			);
			paths.push(func);
			if (type) {
				typeImports.push(type);
			}
			if (responseTypes) {
				typeImports = [...typeImports, ...responseTypes];
			}
		}
	}

	const types = ["export type None = undefined;"];
	for (const [k, v] of Object.entries(data.components.schemas)) {
		const type = buildType(k, v);
		if (type === null) continue;
		types.push(type);
	}

	return {
		paths: paths.toSorted(),
		types: types.toSorted(),
		typeImports: uniq(typeImports)
			.filter((ti) => !ti.endsWith("[]"))
			.toSorted(),
	};
};

const typedFetch = await Deno.readTextFile(
	path.join(__dirname, "templates/typed-fetch.static.ts"),
);

export const generatePaths = async (
	outDir: string,
	paths: { key: string; value: string }[],
	verbose: boolean,
) => {
	VERBOSE = verbose;
	await ensureDir(outDir);
	for (const { key, value } of paths) {
		const start = Temporal.Now.instant();

		console.log(`Generating paths for: ${key}`);
		const { paths, types, typeImports } = await generate(value);

		const pathsFile = await eta.renderAsync("./paths-file", {
			key,
			typeImports,
			paths,
		});

		await Deno.writeTextFile(`${outDir}/paths-${key}.ts`, pathsFile);
		await Deno.writeTextFile(`${outDir}/types-${key}.ts`, types.join("\n\n"));

		const end = Temporal.Now.instant();
		console.log(
			`Generated paths for: ${key} in ${end.since(start).milliseconds}ms`,
		);
	}

	await Deno.writeTextFile(`${outDir}/typed-fetch.ts`, typedFetch);
};
