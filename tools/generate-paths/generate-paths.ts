import { ensureDir } from "https://deno.land/std@0.140.0/fs/mod.ts";
import { Component, Parameter, Route, SwaggerResponse, Type } from "./types.ts";

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
  meta: Route
): { func: string; type: string | null } => {
  method = method.toUpperCase();
  const id = meta.operationId;
  const params = meta.parameters
    ? buildParams(meta.parameters).toLowerCase() + ", "
    : "";
  const url = path.replaceAll("{", "${").toLowerCase();
  const query = meta.parameters
    ? buildQuery(meta.parameters).toLowerCase()
    : "";

  const bodyRef = meta.requestBody?.content["application/json"]?.schema?.$ref;
  const bodyType = bodyRef?.split("/").at(-1);
  const body = bodyType ? `body: ${bodyType}, ` : "";

  const signature = `async (${body}${params}headers?: HeadersInit, options?: RequestInit)`;

  const func = `export const ${id} = ${signature} => await fetch (\`${url}${query}\`, { 
    method: '${method}', 
    headers: { 
      'Content-Type': 'application/json', 
      ...headers 
    },${bodyType ? "\n    body: JSON.stringify(body)," : ""}
    ...options 
  });`;

  return { func, type: bodyType ?? null };
};

const buildType = (name: string, component: Component): string | null => {
  if (!component.properties) return null;

  const typeMappings: { [key: string]: string } = {
    integer: "number",
    undefined: "object",
    array: "object[]",
  };

  let type = `export interface ${name} {\n`;

  for (const [k, v] of Object.entries(component.properties)) {
    const t = typeMappings[v.type] ?? v.type;
    type += `    ${k}: ${t}${v.nullable ? " | null" : ""};\n`;
  }

  type += "}";
  return type;
};

const generate = async (
  path: string
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
  paths: { [key: string]: string }
) => {
  ensureDir(outDir);
  for (const [key, val] of Object.entries(paths)) {
    console.log(`Generating paths for: ${key}`);
    const { paths, types, typeImports } = await generate(val);

    const pathsFile = `import {
\t${typeImports.join(",\n\t")}
} from './types-${key}';\n
${paths.join("\n\n")}`;

    await Deno.writeTextFile(`${outDir}/paths-${key}.ts`, pathsFile);
    await Deno.writeTextFile(`${outDir}/types-${key}.ts`, types.join("\n\n"));
  }
};
