use super::{FunctionView, TypeProperty, render_function, render_paths_file, render_type};

#[test]
fn renders_a_function_without_body() {
    let params = vec!["id: number".to_string(), "q: string | null".to_string()];
    let responses = vec![
        ("200".to_string(), "Foo".to_string()),
        ("404".to_string(), "undefined".to_string()),
    ];
    let rendered = render_function(&FunctionView {
        id: "getFoo",
        params: Some(&params),
        response_type_map: &responses,
        quote: "\"",
        url: "https://example.test/api/foo",
        query: "",
        method: "get",
        body_type: None,
        is_not_empty: false,
    });
    let expected = "\nexport const getFoo = async (id: number, q: string | null, headers?: HeadersInit, options?: RequestInit) => await typedFetch<{ 200: Foo; 404: undefined }, undefined>(\"https://example.test/api/foo\",\n    GET,\n    undefined,\n    headers,\n    options,\n);";
    assert_eq!(rendered, expected);
}

#[test]
fn renders_a_function_with_body_and_interpolated_url() {
    let params = vec!["itemId: number".to_string()];
    let responses = vec![("200".to_string(), "Item".to_string())];
    let rendered = render_function(&FunctionView {
        id: "updateItem",
        params: Some(&params),
        response_type_map: &responses,
        quote: "`",
        url: "https://example.test/api/items/${itemId}",
        query: "?verbose=${_enc(verbose)}",
        method: "patch",
        body_type: Some("ItemUpdate"),
        is_not_empty: true,
    });
    let expected = "\nexport const updateItem = async (body: ItemUpdate, itemId: number, headers?: HeadersInit, options?: RequestInit) => await typedFetch<{ 200: Item }, ItemUpdate>(`https://example.test/api/items/${itemId}?verbose=${_enc(verbose)}`,\n    PATCH,\n    body,\n    headers,\n    options,\n);";
    assert_eq!(rendered, expected);
}

#[test]
fn renders_a_function_without_params() {
    let responses = vec![("204".to_string(), "undefined".to_string())];
    let rendered = render_function(&FunctionView {
        id: "deleteFoo",
        params: None,
        response_type_map: &responses,
        quote: "\"",
        url: "https://example.test/api/foo",
        query: "",
        method: "delete",
        body_type: None,
        is_not_empty: false,
    });
    assert_eq!(
        rendered,
        "\nexport const deleteFoo = async (headers?: HeadersInit, options?: RequestInit) => await typedFetch<{ 204: undefined }, undefined>(\"https://example.test/api/foo\",\n    DELETE,\n    undefined,\n    headers,\n    options,\n);"
    );
}

#[test]
fn renders_an_object_type() {
    let properties = vec![
        TypeProperty {
            name: "id",
            ty: "number",
            description: None,
        },
        TypeProperty {
            name: "name",
            ty: "string | null",
            description: Some("The display name"),
        },
    ];
    assert_eq!(
        render_type(&properties),
        "{\n    id: number;\n    /** The display name */\n    name: string | null;\n}"
    );
}

#[test]
fn renders_a_paths_file() {
    let imports = vec!["Foo".to_string(), "Bar".to_string()];
    let paths = vec![
        "\nexport const a = () => {};".to_string(),
        "\nexport const b = () => {};".to_string(),
    ];
    let rendered = render_paths_file("public", &imports, &paths);
    let expected = "import { DELETE, GET, HEAD, PATCH, POST, PUT, typedFetch } from \"./typed-fetch\";\nimport type {\n\tFoo,\n\tBar,\n} from './types-public';\nconst _enc = (p: string | number | boolean | Date | null | undefined): string => {\n\tif (p === null || p === undefined) return \"\";\n\tif (p instanceof Date) return encodeURIComponent(p.toISOString());\n\treturn encodeURIComponent(typeof p === \"string\" ? p : String(p));\n};\n\n\nexport const a = () => {};\n\nexport const b = () => {};\n";
    assert_eq!(rendered, expected);
}

#[test]
fn renders_a_paths_file_without_imports() {
    let rendered = render_paths_file("private", &[], &[]);
    let expected = "import { DELETE, GET, HEAD, PATCH, POST, PUT, typedFetch } from \"./typed-fetch\";\nimport type {\n} from './types-private';\nconst _enc = (p: string | number | boolean | Date | null | undefined): string => {\n\tif (p === null || p === undefined) return \"\";\n\tif (p instanceof Date) return encodeURIComponent(p.toISOString());\n\treturn encodeURIComponent(typeof p === \"string\" ? p : String(p));\n};\n\n";
    assert_eq!(rendered, expected);
}

#[test]
fn normalises_crlf_to_lf() {
    assert_eq!(
        super::normalize_newlines("a\r\nb\r\n".to_string()),
        "a\nb\n"
    );
    assert_eq!(super::normalize_newlines("a\nb".to_string()), "a\nb");
    assert_eq!(super::normalize_newlines(String::new()), "");
}

#[test]
fn rendered_output_never_contains_carriage_returns() {
    // A Windows checkout may surface the embedded templates with CRLF; the
    // generated TypeScript must still use LF (matching the Eta implementation).
    let properties = vec![TypeProperty {
        name: "id",
        ty: "number",
        description: Some("The id"),
    }];
    assert!(!render_type(&properties).contains('\r'));

    let rendered = render_function(&FunctionView {
        id: "getFoo",
        params: None,
        response_type_map: &[("200".to_string(), "Foo".to_string())],
        quote: "\"",
        url: "https://example.test/api/foo",
        query: "",
        method: "get",
        body_type: None,
        is_not_empty: false,
    });
    assert!(!rendered.contains('\r'));

    assert!(!render_paths_file("public", &[], &[]).contains('\r'));
}
