export interface SwaggerResponse {
	"x-generator": string;
	openapi: string;
	info: { title: string; version: string };
	servers: { url: string; description: string }[];
	paths: Record<string, Path>;
	components: { schemas: { [key: string]: Type } };
}

export type Path = Record<
	"get" | "put" | "post" | "delete" | "head" | "options",
	Route
>;

export interface Route {
	tags: string[];
	operationId: string;
	parameters?: Parameter[];
	requestBody: RequestBody | null;
	responses: Record<string, Response>;
}

export interface Response {
	description: string;
	content: Content;
}

export type ContentType = "application/json" | "application/octet-stream";

export type Content = {
	[K in ContentType]: { [P in K]: { schema: Type } };
}[ContentType];

export interface Parameter {
	name: string;
	in: string;
	required: boolean;
	schema: Type;
}

export interface RequestBody {
	"x-name": string;
	content: Content;
}

export interface Type {
	type: string | string[];
	enum?: string[] | undefined;
	format?: string | undefined;
	nullable?: boolean | undefined;
	properties?: Record<string, Type> | undefined;
	additionalProperties?: Type | undefined;
	items?: Type | undefined;
	oneOf?: Type[] | undefined;
	description?: string | undefined;
	$ref?: string | undefined;
}
