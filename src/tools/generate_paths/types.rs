//! Type definitions mirroring `tools/generate-paths/types.ts`.
//!
//! Maps use `IndexMap` so that iteration order matches JavaScript object
//! insertion order (serde_json is built with the `preserve_order` feature for
//! the same reason).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// `Type["type"]`: OpenAPI allows a single type name or a list of them.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum TypeKind {
    Single(String),
    Multiple(Vec<String>),
}

/// Port of the `Type` interface from `types.ts`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Type {
    #[serde(rename = "type", default)]
    pub ty: Option<TypeKind>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub nullable: Option<bool>,
    #[serde(rename = "enum", default)]
    pub enum_values: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub properties: Option<IndexMap<String, Type>>,
    #[serde(rename = "additionalProperties", default)]
    pub additional_properties: Option<Box<Type>>,
    #[serde(default)]
    pub items: Option<Box<Type>>,
    #[serde(rename = "oneOf", default)]
    pub one_of: Option<Vec<Type>>,
    #[serde(rename = "anyOf", default)]
    pub any_of: Option<Vec<Type>>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "$ref", default)]
    pub reference: Option<String>,
}

/// Port of the `MediaType`-shaped entries used in `Content`.
///
/// `schema` is optional because the original checks `content["application/json"].schema`
/// before deciding whether a request has a body; an entry without a schema is
/// therefore treated as "no body" rather than as an empty schema.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MediaType {
    #[serde(default)]
    pub schema: Option<Type>,
}

/// Port of the `Parameter` interface.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    #[serde(default)]
    pub required: Option<bool>,
    #[serde(default)]
    pub schema: Type,
}

/// Port of the `RequestBody` interface.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestBody {
    #[serde(default)]
    pub content: IndexMap<String, MediaType>,
}

/// Port of the `ApiResponse` interface.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ApiResponse {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub content: Option<IndexMap<String, MediaType>>,
}

/// Port of the `Route` interface. `responses` is required, matching the
/// original tool which throws when a route has no `responses` object.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Route {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(rename = "operationId", default)]
    pub operation_id: Option<String>,
    #[serde(default)]
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "requestBody", default)]
    pub request_body: Option<RequestBody>,
    pub responses: IndexMap<String, ApiResponse>,
}

/// Port of the `SwaggerResponse` interface (only the fields the generator
/// needs).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwaggerResponse {
    pub paths: IndexMap<String, IndexMap<String, Route>>,
    pub components: Components,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Components {
    pub schemas: IndexMap<String, Type>,
}
