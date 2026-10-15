//! Port of `tools/generate-paths/generate-paths.ts`.
//!
//! The generated TypeScript must match the Deno tool byte-for-byte, so the
//! control flow mirrors the original closely (including its quirks, such as
//! `replaceType` mapping `bool` — but not `boolean` — and the left-to-right
//! order of the schema match chain).

pub mod render;
pub mod types;

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::time::Instant;

use anyhow::{Context, Result, bail};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use self::render::{FunctionView, TypeProperty, render_function, render_paths_file, render_type};
use self::types::{ApiResponse, Parameter, Route, SwaggerResponse, Type, TypeKind};
use crate::helpers::js::{camel_case, encode_uri_component, js_to_string, locale_compare};

/// The static `typed-fetch.ts` runtime, copied verbatim.
pub const TYPED_FETCH: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tools/generate-paths/templates/typed-fetch.static.ts"
));

/// `\{.+?\}` from the URL rewriting step.
static URL_GROUP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{(.+?)\}").expect("valid regex"));

/// Maps a primitive OpenAPI type name to its TypeScript equivalent — the
/// `replaceType` table used by `buildParams`.
fn replace_type(ty: &str) -> &'static str {
    match ty {
        "integer" | "decimal" | "double" | "float" => "number",
        "bool" => "boolean",
        "string" => "string",
        "date" => "Date",
        _ => "unknown",
    }
}

/// The `typeMappings` table used *inside* `parseType`. This is deliberately
/// different from [`replace_type`]: `parseType` leaves unknown type names as-is
/// (so `boolean` stays `boolean`), whereas `buildParams` maps them to `unknown`.
fn map_parsed_type(ty: &str) -> &str {
    match ty {
        "integer" => "number",
        "date" => "Date",
        "undefined" => "object",
        "array" => "object[]",
        _ => ty,
    }
}

/// Port of `getTypeActual`.
fn get_type_actual(ty: &Type, verbose: bool) -> (String, Option<bool>) {
    if verbose {
        println!("Getting type actual for {}", json(ty));
    }

    let includes =
        |name: &str| matches!(&ty.ty, Some(TypeKind::Multiple(v)) if v.iter().any(|x| x == name));

    let mut out = String::from("unknown");
    let mut nullable = ty.nullable;

    if ty.format.as_deref() == Some("date-time") {
        out = "date".to_string();
    } else if let Some(TypeKind::Single(kind)) = &ty.ty {
        if kind == "object" && ty.additional_properties.is_some() {
            let (inner, inner_nullable) = get_type_actual(
                ty.additional_properties.as_deref().expect("checked above"),
                verbose,
            );
            out = inner;
            nullable = inner_nullable;
        } else {
            out = kind.clone();
        }
    } else if ty.format.as_deref().is_some_and(|f| f.starts_with("int")) || includes("integer") {
        out = "integer".to_string();
    } else if includes("string") {
        out = "string".to_string();
    }

    if includes("null") {
        nullable = Some(true);
    }

    (out, nullable)
}

/// Port of `parseType`. Returns the rendered type string (possibly empty) and
/// whether importing the type names should be skipped.
fn parse_type(schema: &Type, verbose: bool) -> (String, bool) {
    if verbose {
        println!("Parsing type {}", json(schema));
    }

    let mut nullable = schema.nullable.unwrap_or(false);
    let mut type_strings: Vec<Option<String>>;
    let skip_import: bool;

    if let Some(reference) = &schema.reference {
        type_strings = vec![Some(
            reference
                .rsplit('/')
                .next()
                .unwrap_or(reference)
                .to_string(),
        )];
        skip_import = false;
    } else if schema.format.as_deref() == Some("binary") {
        type_strings = vec![Some("Blob".to_string())];
        skip_import = true;
    } else if let Some(items) = &schema.items {
        let (inner, skip) = parse_type(items, verbose);
        type_strings = vec![Some(format!("{inner}[]"))];
        skip_import = skip;
    } else if let Some(values) = &schema.enum_values {
        type_strings = values
            .iter()
            .map(|value| {
                Some(if js_truthy(value) {
                    format!("\"{}\"", js_to_string(value))
                } else {
                    "null".to_string()
                })
            })
            .collect();
        skip_import = true;
    } else if let Some(members) = &schema.one_of {
        let mut subtypes = Vec::new();
        for member in members {
            let (parsed, _) = parse_type(member, verbose);
            if !parsed.is_empty() {
                subtypes.push(Some(parsed));
            }
        }
        type_strings = subtypes;
        skip_import = true;
    } else if let Some(members) = &schema.any_of {
        let is_nullable = members
            .iter()
            .any(|member| matches!(&member.ty, Some(TypeKind::Single(t)) if t == "null"));
        let mut subtypes = Vec::new();
        for member in members {
            if matches!(&member.ty, Some(TypeKind::Single(t)) if t == "null") {
                continue;
            }
            let (parsed, _) = parse_type(member, verbose);
            if !parsed.is_empty() {
                subtypes.push(Some(parsed));
            }
        }
        if is_nullable {
            nullable = true;
        }
        type_strings = subtypes;
        skip_import = false;
    } else if let Some(properties) = &schema.properties {
        let mut owned: Vec<(String, String, Option<String>)> = Vec::new();
        for (name, value) in properties {
            let (parsed, _) = parse_type(value, verbose);
            if parsed.is_empty() {
                continue;
            }
            owned.push((name.clone(), parsed, value.description.clone()));
        }
        let props: Vec<TypeProperty<'_>> = owned
            .iter()
            .map(|(name, ty, description)| TypeProperty {
                name,
                ty,
                description: description.as_deref(),
            })
            .collect();
        type_strings = vec![Some(render_type(&props))];
        skip_import = true;
    } else if schema.ty.is_some() {
        let (type_name, type_nullable) = get_type_actual(schema, verbose);
        nullable = type_nullable.unwrap_or(false);
        type_strings = vec![Some(map_parsed_type(&type_name).to_string())];
        skip_import = true;
    } else {
        type_strings = vec![None];
        skip_import = true;
    }

    if nullable {
        type_strings.push(Some("null".to_string()));
    }

    // `uniq(compact(typeStrings))` — drop nulls/undefined and duplicates while
    // preserving the first occurrence.
    let mut seen = HashSet::new();
    let mut unique: Vec<String> = type_strings
        .into_iter()
        .flatten()
        .filter(|value| seen.insert(value.clone()))
        .collect();
    // `toSorted` with `null` forced last and `localeCompare` elsewhere.
    unique.sort_by(|a, b| match (a == "null", b == "null") {
        (true, true) => std::cmp::Ordering::Equal,
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        (false, false) => locale_compare(a, b),
    });

    if verbose {
        println!(
            "Parsed type ({}nullable) {:?} {}",
            if nullable { "" } else { "not " },
            unique,
            skip_import
        );
    }

    (unique.join(" | "), skip_import)
}

/// Builds the parameter list for a function signature (`buildParams`).
fn build_params(parameters: &[Parameter], current_function: &str, verbose: bool) -> Vec<String> {
    if verbose {
        println!("Building params for {current_function}");
    }

    let mut params = Vec::new();
    for parameter in parameters {
        let (ty, _) = get_type_actual(&parameter.schema, verbose);
        let nullable = !parameter.required.unwrap_or(false);

        if verbose {
            println!("Param {} {} {}", json(parameter), ty, nullable);
        }

        if !matches!(
            parameter.location.as_str(),
            "path" | "query" | "querystring"
        ) {
            continue;
        }
        params.push(format!(
            "{}: {}{}",
            camel_case(&parameter.name),
            replace_type(&ty),
            if nullable { " | null" } else { "" }
        ));
    }
    params
}

/// Builds the query string fragment (`buildQuery`).
fn build_query(parameters: &[Parameter]) -> String {
    let mut params = Vec::new();
    for parameter in parameters {
        if !matches!(parameter.location.as_str(), "query" | "querystring") {
            continue;
        }
        let name = camel_case(&parameter.name);
        params.push(format!("{name}=${{_enc({name})}}"));
    }
    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    }
}

/// Rewrites `/items/{itemId}` into `/items/${itemId}`.
fn transform_url(path: &str) -> String {
    let replaced = URL_GROUP.replace_all(path, |captures: &regex::Captures<'_>| {
        format!("{{{}}}", camel_case(&captures[1]))
    });
    replaced.replace('{', "${")
}

/// Port of `buildResponseType`.
fn build_response_type(response: &ApiResponse, verbose: bool) -> (Option<String>, bool) {
    let Some(content) = &response.content else {
        return (None, true);
    };

    if let Some(media) = content.get("application/json") {
        return parse_media(media, verbose);
    }
    if let Some(media) = content.get("application/octet-stream") {
        return parse_media(media, verbose);
    }
    if let Some(media) = content.get("text/plain") {
        return parse_media(media, verbose);
    }
    if content.contains_key("text/event-stream") {
        return (Some("ReadableStream<string>".to_string()), true);
    }
    if let Some(media) = content.get("application/jsonl") {
        let (ty, _) = parse_media(media, verbose);
        return (
            Some(format!("ReadableStream<{}>", ty.unwrap_or_default())),
            true,
        );
    }
    if let Some(media) = content.get("application/json-seq") {
        let (ty, _) = parse_media(media, verbose);
        return (
            Some(format!("ReadableStream<{}>", ty.unwrap_or_default())),
            true,
        );
    }

    (None, true)
}

/// Parses the schema of a media type entry that is known to be present. A
/// missing schema is treated as an empty type (the original would throw when
/// dereferencing `schema`).
fn parse_media(
    media: &crate::tools::generate_paths::types::MediaType,
    verbose: bool,
) -> (Option<String>, bool) {
    match &media.schema {
        Some(schema) => {
            let (ty, skip) = parse_type(schema, verbose);
            (Some(ty), skip)
        }
        None => (Some(String::new()), true),
    }
}

struct FunctionResult {
    func: String,
    body_type: Option<String>,
    response_types: Vec<String>,
}

/// Port of `buildFunction`.
fn build_function(path: &str, method: &str, meta: &Route, verbose: bool) -> FunctionResult {
    let id = meta.operation_id.clone().unwrap_or_default();

    if verbose {
        println!("Building function for {} {}", id, json(meta));
    }

    let params = meta
        .parameters
        .as_ref()
        .map(|p| build_params(p, &id, verbose));
    let url = transform_url(path);
    let query = meta
        .parameters
        .as_ref()
        .map(|p| build_query(p))
        .unwrap_or_default();
    let quote = if (path.contains('{') && path.contains('}'))
        || (query.contains('{') && query.contains('}'))
    {
        "`"
    } else {
        "\""
    };

    let body_schema = meta
        .request_body
        .as_ref()
        .and_then(|body| body.content.get("application/json"))
        .and_then(|media| media.schema.as_ref());
    let body_type = body_schema.map(|schema| parse_type(schema, verbose).0);
    let is_not_empty = body_type.as_deref().is_some_and(|ty| !ty.is_empty());

    let mut response_type_map: Vec<(String, String)> = Vec::new();
    let mut response_types: Vec<String> = Vec::new();
    for (status, response) in &meta.responses {
        let (ty, skip_import) = build_response_type(response, verbose);
        response_type_map.push((
            status.clone(),
            ty.clone().unwrap_or_else(|| "undefined".to_string()),
        ));
        if !skip_import
            && let Some(ty) = &ty
            && !ty.is_empty()
        {
            for name in ty
                .split(" | ")
                .map(|part| part.replacen("[]", "", 1).trim().to_string())
            {
                if ![
                    "null",
                    "undefined",
                    "never",
                    "string",
                    "bool",
                    "number",
                    "object",
                    "array",
                ]
                .contains(&name.as_str())
                    && !name.starts_with('"')
                {
                    response_types.push(name);
                }
            }
        }
    }

    if verbose {
        println!(
            "TYPEMAP [{}]",
            response_type_map
                .iter()
                .map(|(status, ty)| format!("{{ status: \"{status}\", type: \"{ty}\" }}"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    let func = render_function(&FunctionView {
        id: &id,
        params: params.as_deref(),
        response_type_map: &response_type_map,
        quote,
        url: &url,
        query: &query,
        method,
        body_type: body_type.as_deref(),
        is_not_empty,
    });

    FunctionResult {
        func,
        body_type,
        response_types,
    }
}

/// Port of `buildType`.
fn build_type(name: &str, component: &Type, verbose: bool) -> String {
    let (ty, _) = parse_type(component, verbose);
    if ty.is_empty() {
        format!("export type {name} = Record<string, never>;")
    } else {
        format!("export type {name} = {ty};")
    }
}

/// Result of generating one OpenAPI document.
pub struct GeneratedPaths {
    pub paths: Vec<String>,
    pub types: Vec<String>,
    pub type_imports: Vec<String>,
}

/// Port of `generate`: loads a single OpenAPI document and renders it.
pub fn generate(url: &str, cache: bool, insecure: bool, verbose: bool) -> Result<GeneratedPaths> {
    let value = load_spec_value(url, cache, insecure)?;
    generate_value(value, verbose)
        .with_context(|| format!("Failed to parse OpenAPI spec from {url}"))
}

/// Renders an already-parsed OpenAPI document (the body of `generate`).
pub fn generate_value(value: Value, verbose: bool) -> Result<GeneratedPaths> {
    let spec: SwaggerResponse = serde_json::from_value(value)?;

    let mut paths = Vec::new();
    let mut type_imports: Vec<String> = Vec::new();

    for (path, item) in &spec.paths {
        for (method, route) in item {
            let result = build_function(path, method, route, verbose);
            paths.push(result.func);
            if let Some(body_type) = result.body_type
                && !body_type.is_empty()
            {
                type_imports.push(body_type);
            }
            type_imports.extend(result.response_types);
        }
    }

    let mut types = vec!["export type None = undefined;".to_string()];
    for (name, component) in &spec.components.schemas {
        types.push(build_type(name, component, verbose));
    }

    paths.sort();
    types.sort();

    // `uniq(typeImports.map((ti) => ti.replace("[]", ""))).toSorted()`
    let mut seen = HashSet::new();
    let mut type_imports: Vec<String> = type_imports
        .iter()
        .map(|ty| ty.replacen("[]", "", 1))
        .filter(|ty| seen.insert(ty.clone()))
        .collect();
    type_imports.sort();

    Ok(GeneratedPaths {
        paths,
        types,
        type_imports,
    })
}

/// Port of `generatePaths`: renders every supplied document into `out_dir`.
pub fn generate_paths(
    out_dir: &str,
    specs: &[(String, String)],
    verbose: bool,
    cache: bool,
    insecure: bool,
) -> Result<()> {
    fs::create_dir_all(out_dir)
        .with_context(|| format!("Failed to create output directory {out_dir}"))?;

    for (key, value) in specs {
        let start = Instant::now();
        println!("Generating paths for: {key}");

        let generated = generate(value, cache, insecure, verbose)?;
        let paths_file = render_paths_file(key, &generated.type_imports, &generated.paths);

        fs::write(
            PathBuf::from(out_dir).join(format!("paths-{key}.ts")),
            paths_file,
        )?;
        fs::write(
            PathBuf::from(out_dir).join(format!("types-{key}.ts")),
            generated.types.join("\n\n"),
        )?;

        println!(
            "Generated paths for: {key} in {}ms",
            start.elapsed().as_millis()
        );
    }

    fs::write(PathBuf::from(out_dir).join("typed-fetch.ts"), TYPED_FETCH)?;
    Ok(())
}

fn load_spec_value(url: &str, cache: bool, insecure: bool) -> Result<Value> {
    match fetch_text(url, insecure) {
        Ok(text) => {
            let value: Value = serde_json::from_str(&text)?;
            if cache {
                write_cache(url, &value)?;
            }
            Ok(value)
        }
        Err(fetch_error) => {
            if !cache {
                bail!("Failed to fetch OpenAPI spec from {url}: {fetch_error}");
            }
            let cache_path = cache_path(url);
            let text = fs::read_to_string(&cache_path).map_err(|cache_error| {
                anyhow::anyhow!(
                    "Failed to fetch OpenAPI spec from {url} and no cache available: {cache_error}"
                )
            })?;
            eprintln!("Using cached OpenAPI spec from {}", cache_path.display());
            let value: Value = serde_json::from_str(&text).map_err(|cache_error| {
                anyhow::anyhow!(
                    "Failed to fetch OpenAPI spec from {url} and no cache available: {cache_error}"
                )
            })?;
            Ok(value)
        }
    }
}

fn fetch_text(url: &str, insecure: bool) -> Result<String> {
    let mut builder = ureq::Agent::config_builder()
        // Deno's `fetch` does not reject non-2xx responses; HTTP errors surface
        // later when the body is parsed.
        .http_status_as_error(false);
    if insecure {
        builder = builder.tls_config(
            ureq::tls::TlsConfig::builder()
                .disable_verification(true)
                .build(),
        );
    }
    let agent: ureq::Agent = builder.build().into();

    let body = agent.get(url).call()?.body_mut().read_to_string()?;
    Ok(body)
}

/// Cache location: `.cache/<encodeURIComponent(url)>.json` relative to the
/// current working directory (the original located it next to the script).
fn cache_path(url: &str) -> PathBuf {
    let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    base.join(".cache")
        .join(format!("{}.json", encode_uri_component(url)))
}

fn write_cache(url: &str, value: &Value) -> Result<()> {
    let path = cache_path(url);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_string(value)?)
        .with_context(|| format!("Failed to write cache file {}", path.display()))?;
    Ok(())
}

fn json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap_or_default()
}

/// JavaScript truthiness for JSON values.
fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(number) => number.as_f64().is_none_or(|number| number != 0.0),
        Value::String(text) => !text.is_empty(),
        _ => true,
    }
}

#[cfg(test)]
mod tests;
