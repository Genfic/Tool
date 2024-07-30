export interface SwaggerResponse {
	"x-generator": string;
	openapi: string;
	info: { title: string; version: string };
	paths: Map<string, Path>;
	components: { schemas: Map<string, Component> };
}

export type Path = Map<
	"get" | "put" | "post" | "delete" | "head" | "options",
	Route
>;

export interface Route {
	tags: string[];
	operationId: string;
	parameters?: Map<number, Parameter>;
	requestBody: RequestBody | null;
	responses: Map<string, Response>;
}

export interface Response {
	description: string;
	content:
		| { "application/json": { schema: Schema } }
		| { "application/octet-stream": { schema: Schema } };
}

export type Schema =
	| { $ref: string }
	| { type: string; nullable: boolean }
	| { type: string; items: { $ref: string } | { type: string } };

export interface Parameter {
	name: string;
	in: string;
	schema: { type: string; nullable: boolean };
}

export interface RequestBody {
	"x-name": string;
	content: { "application/json": { schema: { $ref: string } } };
}

export interface Component {
	enum: Map<number, string>;
	properties: Map<string, Type>;
}

export interface Type {
	type: string;
	format: string | undefined;
	nullable: boolean | undefined;
	items: { type: string };
	$ref: string;
}
