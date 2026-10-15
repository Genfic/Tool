use super::*;

fn schema(json: &str) -> Type {
    serde_json::from_str(json).expect("valid schema")
}

#[test]
fn map_parsed_type_matches_the_original_mapping() {
    assert_eq!(map_parsed_type("integer"), "number");
    assert_eq!(map_parsed_type("date"), "Date");
    assert_eq!(map_parsed_type("undefined"), "object");
    assert_eq!(map_parsed_type("array"), "object[]");
    // Unknown names are left as-is (unlike `replace_type`).
    assert_eq!(map_parsed_type("boolean"), "boolean");
    assert_eq!(map_parsed_type("number"), "number");
    assert_eq!(map_parsed_type("unknown"), "unknown");
}

#[test]
fn parses_refs_items_enums_and_primitives() {
    assert_eq!(
        parse_type(&schema(r##"{"$ref":"#/components/schemas/Foo"}"##), false).0,
        "Foo"
    );
    assert_eq!(
        parse_type(
            &schema(r##"{"type":"array","items":{"$ref":"#/components/schemas/Foo"}}"##),
            false
        )
        .0,
        "Foo[]"
    );
    // Falsy enum members become "null", which is then sorted last.
    assert_eq!(
        parse_type(&schema(r#"{"type":"string","enum":["a","","b"]}"#), false).0,
        "\"a\" | \"b\" | null"
    );
    assert_eq!(
        parse_type(&schema(r#"{"format":"binary","type":"string"}"#), false).0,
        "Blob"
    );
    assert_eq!(
        parse_type(&schema(r#"{"type":"string","format":"date-time"}"#), false).0,
        "Date"
    );
    // `boolean` stays `boolean` inside `parseType` (see `map_parsed_type`).
    assert_eq!(
        parse_type(&schema(r#"{"type":"boolean"}"#), false).0,
        "boolean"
    );
    assert_eq!(
        parse_type(&schema(r#"{"type":"object"}"#), false).0,
        "object"
    );
}

#[test]
fn parses_one_of_and_any_of() {
    let one_of = schema(r##"{"oneOf":[{"$ref":"#/components/schemas/Foo"},{"type":"string"}]}"##);
    assert_eq!(parse_type(&one_of, false).0, "Foo | string");
    assert!(parse_type(&one_of, false).1);

    let any_of = schema(r#"{"anyOf":[{"type":"string"},{"type":"null"}]}"#);
    assert_eq!(parse_type(&any_of, false).0, "string | null");
    assert!(!parse_type(&any_of, false).1);
}

#[test]
fn parses_inline_objects() {
    let object = schema(
        r#"{"type":"object","properties":{"id":{"type":"integer"},"name":{"type":"string","nullable":true,"description":"The name"}}}"#,
    );
    assert_eq!(
        parse_type(&object, false).0,
        "{\n    id: number;\n    /** The name */\n    name: string | null;\n}"
    );
}

#[test]
fn sorts_union_members_case_insensitively_with_null_last() {
    let union = schema(
        r##"{"oneOf":[{"$ref":"#/components/schemas/Url"},{"type":"integer"},{"type":"string"}]}"##,
    );
    // `["Url", "number", "string"].sort((a, b) => a.localeCompare(b))`
    // yields `["number", "string", "Url"]` in the ICU default locale.
    assert_eq!(parse_type(&union, false).0, "number | string | Url");
}

#[test]
fn transforms_urls_and_queries() {
    assert_eq!(
        transform_url("/api/items/{item-id}/parts/{part_id}"),
        "/api/items/${itemId}/parts/${partId}"
    );
    assert_eq!(transform_url("/api/items"), "/api/items");

    let params = vec![
        Parameter {
            name: "item-id".to_string(),
            location: "path".to_string(),
            required: Some(true),
            schema: schema(r#"{"type":"integer"}"#),
        },
        Parameter {
            name: "search_term".to_string(),
            location: "query".to_string(),
            required: None,
            schema: schema(r#"{"type":"string"}"#),
        },
    ];
    assert_eq!(
        build_query(&params),
        "?searchTerm=${_enc(searchTerm ?? \"\")}"
    );
    assert_eq!(
        build_params(&params, "f", false),
        (
            vec![
                "itemId: number".to_string(),
                "searchTerm: string | null".to_string()
            ],
            vec![]
        )
    );
}

#[test]
fn builds_response_types() {
    let response: ApiResponse = serde_json::from_str(
        r##"{"content":{"application/json":{"schema":{"$ref":"#/components/schemas/Foo"}}}}"##,
    )
    .unwrap();
    assert_eq!(
        build_response_type(&response, false),
        (Some("Foo".to_string()), false)
    );

    let stream: ApiResponse = serde_json::from_str(
        r##"{"content":{"application/jsonl":{"schema":{"$ref":"#/components/schemas/Foo"}}}}"##,
    )
    .unwrap();
    assert_eq!(
        build_response_type(&stream, false),
        (Some("ReadableStream<Foo>".to_string()), true)
    );

    let sse: ApiResponse =
        serde_json::from_str(r#"{"content":{"text/event-stream":{"schema":{"type":"string"}}}}"#)
            .unwrap();
    assert_eq!(
        build_response_type(&sse, false),
        (Some("ReadableStream<string>".to_string()), true)
    );

    let empty: ApiResponse = serde_json::from_str(r#"{"description":"ok"}"#).unwrap();
    assert_eq!(build_response_type(&empty, false), (None, true));
}

#[test]
fn builds_component_types() {
    assert_eq!(
        build_type("Foo", &schema(r#"{"type":"string"}"#), false),
        "export type Foo = string;"
    );
    assert_eq!(
        build_type("Empty", &schema(r#"{}"#), false),
        "export type Empty = Record<string, never>;"
    );
}

#[test]
fn builds_a_function_with_body_params_and_responses() {
    let route: Route = serde_json::from_value(serde_json::json!({
            "operationId": "getItem",
            "parameters": [
                {"name": "itemId", "in": "path", "required": true, "schema": {"type": "integer"}}
            ],
            "requestBody": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Item"}}}},
            "responses": {
                "200": {"content": {"application/json": {"schema": {"$ref": "#/components/schemas/Item"}}}},
                "404": {"description": "not found"}
            }
        }))
        .unwrap();

    let result = build_function("/api/items/{itemId}", "get", &route, false);
    assert_eq!(result.body_type.as_deref(), Some("Item"));
    assert_eq!(result.response_types, vec!["Item".to_string()]);
    // The path contains `{}`, so the URL is written as a template literal.
    assert_eq!(
        result.func,
        "\nexport const getItem = async (body: Item, itemId: number, headers?: HeadersInit, options?: RequestInit) => await typedFetch<{ 200: Item; 404: undefined }, Item>(`/api/items/${itemId}`,\n    GET,\n    body,\n    headers,\n    options,\n);"
    );
}

#[test]
fn a_request_body_without_a_schema_becomes_undefined() {
    let route: Route = serde_json::from_value(serde_json::json!({
        "operationId": "ping",
        "requestBody": {"content": {"application/json": {}}},
        "responses": {"204": {"description": "no content"}}
    }))
    .unwrap();

    let result = build_function("/ping", "post", &route, false);
    assert_eq!(result.body_type, None);
    assert!(
        result
            .func
            .contains("typedFetch<{ 204: undefined }, undefined>")
    );
    assert!(result.func.contains(",\n    undefined,\n"));
}

#[test]
fn generate_renders_paths_types_and_imports() {
    let value = serde_json::json!({
        "paths": {
            "/items": {
                "get": {
                    "operationId": "listItems",
                    "responses": {
                        "200": {"content": {"application/json": {"schema": {"type": "array", "items": {"$ref": "#/components/schemas/Item"}}}}}
                    }
                }
            }
        },
        "components": {"schemas": {"Item": {"type": "object", "properties": {"id": {"type": "integer"}}}}}
    });

    let generated = generate_value(value, false).unwrap();
    assert_eq!(generated.type_imports, vec!["Item".to_string()]);
    assert_eq!(
        generated.types,
        vec![
            "export type Item = {\n    id: number;\n};".to_string(),
            "export type None = undefined;".to_string(),
        ]
    );
    assert_eq!(generated.paths.len(), 1);
}
