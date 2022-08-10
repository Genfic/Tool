export interface SwaggerResponse {
	'x-generator': string;
	openapi: string;
	info: { title: string; version: string };
	paths: Map<string, Path>;
	components: { schemas: Map<string, Component> };
}

export type Path = Map<string, Route>;

export interface Route {
	tags: string[];
	operationId: string;
	parameters?: Map<number, Parameter>;
	responses?: Map<number, ResponseObject>;
	requestBody: RequestBody | null;
}

export interface Parameter {
	name: string;
	in: string;
	schema: { type: string; nullable: boolean };
}

interface ResponseObject {
	description: string;
	content: { 'application/json': { schema: Schema } };
}

export type Schema = { $ref: string } | { type: string } | { type: 'array', items: { $ref?: string, type?: string } };

export interface RequestBody {
	'x-name': string;
	content: { 'application/json': { schema: { $ref: string } } };
}

export interface Component {
	enum: Map<number, string>;
	properties: Map<string, Type>;
}

export interface Type {
	type: string;
	format: string | undefined;
	nullable: boolean | undefined;
	$ref: string;
}
