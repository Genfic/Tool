import { ensureDir } from 'https://deno.land/std@0.140.0/fs/mod.ts';
import { Parameter, Route, SwaggerResponse } from './types.ts';

const replaceType = (type: string): string => {
	return (
		{
			integer: 'number',
			decimal: 'number',
			double: 'number',
			float: 'number',
			bool: 'boolean',
			string: 'string',
		}[type] ?? 'any'
	);
};

const buildParams = (parameters: Map<number, Parameter>): string => {
	const params = [];
	for (const [_, param] of Object.entries(parameters)) {
		if (param.in !== 'path' && param.in !== 'query') continue;
		let p = param.name;
		p += ': ';
		p += replaceType(param.schema.type);
		params.push(p);
	}
	return params.join(', ');
};

const buildQuery = (parameters: Map<number, Parameter>): string => {
	const params = [];
	for (const [_, param] of Object.entries(parameters)) {
		if (param.in !== 'query') continue;
		let p = param.name;
		p += '=${';
		p += param.name;
		p += '}';
		params.push(p);
	}
	return params.length > 0 ? `?${params.join('&')}` : '';
};

const buildFunction = (path: string, method: string, meta: Route) => {
	method = method.toUpperCase();
	const id = meta.operationId;
	const params = meta.parameters ? buildParams(meta.parameters).toLowerCase() + ', ' : '';
	const url = path.replaceAll('{', '${').toLowerCase();
	const query = meta.parameters ? buildQuery(meta.parameters).toLowerCase() : '';

	const signature = `async (${params}headers?: HeadersInit, options?: RequestInit)`;

	return `export const ${id} = ${signature} => await fetch (\`${url}${query}\`, { 
    method: '${method}', 
    headers: { 
      'Content-Type': 'application/json', 
      ...headers 
    }, 
    ...options 
  });`;
};

const generate = async (path: string): Promise<string[]> => {
	const res = await fetch(path);
	const data: SwaggerResponse = await res.json();

	const paths = [];

	for (const [k, v] of Object.entries(data.paths)) {
		for (const [pathKey, pathVal] of Object.entries(v)) {
			const fun = buildFunction(k, pathKey, pathVal as Route);
			paths.push(fun);
		}
	}

	return paths;
};

export const generatePaths = async (
	outDir: string,
	paths: { [key: string]: string },
) => {
	ensureDir(outDir);
	for (const [key, val] of Object.entries(paths)) {
		console.log(`Generating paths for: ${key}`);
		const paths = await generate(val);
		await Deno.writeTextFile(`${outDir}/paths-${key}.ts`, paths.join('\n\n'));
	}
};
