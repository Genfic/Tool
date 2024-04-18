import { ensureDir } from "https://deno.land/std@0.140.0/fs/mod.ts";
import type { Component, Parameter, Route, SwaggerResponse } from "./types.ts";

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

const buildFunction = (
	path: string,
	method: string,
	meta: Route,
): { func: string; type: string | null } => {
	const id = meta.operationId;
	const params = meta.parameters
		? `${buildParams(meta.parameters).toLowerCase()}, `
		: "";
	const url = path.replaceAll("{", "${").toLowerCase();
	const query = meta.parameters
		? buildQuery(meta.parameters).toLowerCase()
		: "";

	// If the path or the query contains `{}`s, that means the resulting string has to use an interpolated string
	const q = [path, query].some((s) => ["{", "}"].every((c) => s.includes(c)))
		? "`"
		: '"';

	const bodyRef = meta.requestBody?.content["application/json"]?.schema?.$ref;
	const bodyType = bodyRef?.split("/").at(-1);
	const body = bodyType ? `body: ${bodyType}, ` : "";

	const signature = `async (${body}${params}headers?: HeadersInit, options?: RequestInit)`;

	const func = `export const ${id} = ${signature} => await fetch (${q}${url}${query}${q}, { 
    method: '${method.toUpperCase()}', 
    headers: { 
      'Content-Type': 'application/json', 
      ...headers 
    },${bodyType ? "\n    body: JSON.stringify(body)," : ""}
    ...options 
  });`;

	return { func, type: bodyType ?? null };
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
					variableType =
						typeMappings[meta.items.type] ?? `${meta.items.type}[]`;
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

		for (const [propertyName, propertyMetadata] of Object.entries(
			component.properties,
		)) {
			const variableType = constructType(propertyMetadata);

			type += `    ${propertyName}: ${variableType};\n`;
		}

		type += "}";

		return type;
	}

	if (component.enum) {
		const type = `export type ${name} = ${[...component.enum.values()]
			.map((e) => `"${e}"`)
			.join(" | ")};`;
		return type;
	}

	return null;
};

const generate = async (
	path: string,
): Promise<{
	paths: string[];
	types: string[];
	typeImports: (string | null)[];
}> => {
	const res = await fetch(path);
	const data: SwaggerResponse = await res.json();

	const paths = [];
	const typeImports: (string | null)[] = [];

	for (const [k, v] of Object.entries(data.paths)) {
		for (const [pathKey, pathVal] of Object.entries(v)) {
			const { func, type } = buildFunction(k, pathKey, pathVal as Route);
			paths.push(func);
			if (type) typeImports.push(type);
		}
	}

	const types = [];
	for (const [k, v] of Object.entries(data.components.schemas)) {
		const type = buildType(k, v);
		if (type === null) continue;
		types.push(type);
	}

	return { paths, types, typeImports };
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
\t${typeImports.join(",\n\t")}
} from './types-${key}';\n
${paths.join("\n\n")}`;

		await Deno.writeTextFile(`${outDir}/paths-${key}.ts`, pathsFile);
		await Deno.writeTextFile(`${outDir}/types-${key}.ts`, types.join("\n\n"));
	}
};
