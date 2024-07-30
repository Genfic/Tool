import { ensureDir } from "@std/fs";
import { uniq } from "@es-toolkit/es-toolkit";

import type { Component, Parameter, Response as Res, Route, Schema, SwaggerResponse } from "./types.ts";
import { typedFetchText } from "./static.ts";

const replaceType = (type: string): string => {
	return (
		{
			integer: "number",
			decimal: "number",
			double: "number",
			float: "number",
			bool: "boolean",
			string: "string",
		}[type] ?? "any"
	);
};

const buildParams = (parameters: Map<number, Parameter>): string => {
	const params = [];
	for (const [_, param] of Object.entries(parameters)) {
		if (param.in !== "path" && param.in !== "query") continue;
		let p = param.name;
		p += ": ";
		p += replaceType(param.schema.type);
		params.push(p);
	}
	return params.join(", ");
};

const buildQuery = (parameters: Map<number, Parameter>): string => {
	const params = [];
	for (const [_, param] of Object.entries(parameters)) {
		if (param.in !== "query") continue;
		let p = param.name;
		p += "=${";
		p += param.name;
		p += "}";
		params.push(p);
	}
	return params.length > 0 ? `?${params.join("&")}` : "";
};

const parseType = (schema: Schema): [string | undefined, boolean] => {
	const typeMappings: { [key: string]: string } = {
		integer: "number",
		undefined: "object",
		array: "object[]",
	};

	const extract = (ref: string) => ref.split("/").at(-1);

	if ("$ref" in schema) {
		return [extract(schema.$ref), false];
	}
	if (
		"type" in schema &&
		schema.type === "array" &&
		"items" in schema &&
		schema.items
	) {
		if ("$ref" in schema.items && schema.items.$ref) {
			return [`${extract(schema.items.$ref)}[]`, false];
		}
		if ("type" in schema.items && schema.items.type) {
			return [
				`${typeMappings[schema.items.type] ?? schema.items.type}[]`,
				true,
			];
		}
	}
	if (
		"type" in schema &&
		schema.type &&
		"nullable" in schema &&
		schema.nullable
	) {
		return [
			`${typeMappings[schema.type]}${schema.nullable ? " | null" : ""}`,
			true,
		];
	}
	if ("type" in schema && schema.type) {
		return [typeMappings[schema.type] ?? schema.type, true];
	}

	return [undefined, true];
};

const buildResponseType = (response: Res): [string | undefined, boolean] => {
	if (!response.content) return [undefined, true];

	const schema = "application/json" in response.content
		? response.content["application/json"].schema
		: response.content["application/octet-stream"].schema;

	return parseType(schema);
};

const buildFunction = (
	path: string,
	method: string,
	meta: Route,
): {
	func: string;
	type: string | null;
	responseTypes: string[];
} => {
	const id = meta.operationId;
	const params = meta.parameters ? `${buildParams(meta.parameters).toLowerCase()}, ` : "";
	const url = path.replaceAll("{", "${").toLowerCase();
	const query = meta.parameters ? buildQuery(meta.parameters).toLowerCase() : "";

	// If the path or the query contains `{}`s, that means the resulting string has to use an interpolated string
	const q = [path, query].some((s) => ["{", "}"].every((c) => s.includes(c))) ? "`" : '"';

	const bodyRef = meta.requestBody?.content["application/json"]?.schema;

	const [bodyType, _] = bodyRef ? parseType(bodyRef) : [undefined, false];
	const body = bodyType ? `body: ${bodyType}, ` : "";

	const responseTypes: [string | undefined, boolean][] = [];
	for (const [_, response] of Object.entries(meta.responses)) {
		const t = buildResponseType(response);
		if (t) {
			responseTypes.push(t);
		}
	}

	const responseType: string = responseTypes.every(([t, _]) => t === undefined)
		? "void"
		: uniq(responseTypes.map(([t, _]) => t).filter((t) => !!t)).join("|");

	const signature = `async (${body}${params}headers?: HeadersInit, options?: RequestInit)`;

	const func = `export const ${id} = ${signature} => await typedFetch<${responseType}>(${q}${url}${query}${q}, 
    '${method.toUpperCase()}', 
    ${bodyType ? "body" : "undefined"},
    headers,
    options,
  );`;

	return {
		func,
		type: bodyType ?? null,
		responseTypes: responseTypes
			.filter(([_, p]) => !p)
			.map(([t, _]) => t?.replace("[]", ""))
			.filter((x) => !!x) as string[],
	};
};

const buildType = (name: string, component: Component): string | null => {
	if (component.properties) {
		const typeMappings: { [key: string]: string } = {
			integer: "number",
			undefined: "object",
			array: "object[]",
		};

		let type = `export interface ${name} {\n`;

		type meta = {
			type: string;
			nullable: boolean;
			items: meta;
			$ref: string;
			oneOf: meta[];
		};
		const constructType = (meta: meta): string | null => {
			let variableType = "";
			if (meta.type) {
				if (meta.type === "array" && meta.items) {
					variableType = typeMappings[meta.items.type] ?? `${meta.items.type}[]`;
				} else {
					variableType = typeMappings[meta.type] ?? meta.type;
				}
			} else if (meta.$ref) {
				variableType = meta.$ref.split("/").at(-1) ?? "unknown";
			} else if (meta.oneOf) {
				let subtype = "";
				for (const t of meta.oneOf) {
					subtype += constructType(t);
				}
				variableType = subtype;
			} else {
				return null;
			}

			if (meta.nullable) {
				variableType += " | null";
			}

			return variableType;
		};

		for (
			const [propertyName, propertyMetadata] of Object.entries(
				component.properties,
			)
		) {
			const variableType = constructType(propertyMetadata);

			type += `    ${propertyName}: ${variableType};\n`;
		}

		type += "}";

		return type;
	}

	if (component.enum) {
		return `export type ${name} = ${
			[...component.enum.values()]
				.map((e) => `"${e}"`)
				.join(" | ")
		};`;
	}

	return null;
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

	return { paths, types, typeImports: uniq(typeImports) };
};

export const generatePaths = async (
	outDir: string,
	paths: { [key: string]: string },
) => {
	await ensureDir(outDir);
	for (const [key, val] of Object.entries(paths)) {
		console.log(`Generating paths for: ${key}`);
		const { paths, types, typeImports } = await generate(val);

		const pathsFile = `import type {
\t${typeImports.filter((ti) => !ti.endsWith("[]")).join(",\n\t")}
} from './types-${key}';
import { typedFetch } from './typed-fetch';\n
${paths.join("\n\n")}`;

		await Deno.writeTextFile(`${outDir}/paths-${key}.ts`, pathsFile);
		await Deno.writeTextFile(`${outDir}/types-${key}.ts`, types.join("\n\n"));
	}

	await Deno.writeTextFile(`${outDir}/typed-fetch.ts`, typedFetchText);
};
