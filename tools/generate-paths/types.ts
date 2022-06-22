export interface SwaggerResponse {
  "x-generator": string;
  openapi: string;
  info: { title: string; version: string };
  paths: Map<string, Path>;
}

export type Path = Map<string, Route>;

export interface Route {
  tags: string[];
  operationId: string;
  parameters?: Map<number, Parameter>;
}

export interface Parameter {
  name: string;
  in: string;
  schema: { type: string; nullable: boolean };
}
