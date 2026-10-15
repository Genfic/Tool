//! Jinja-based rendering of the generated TypeScript files.
//!
//! The `.jinja` templates live in `templates/` and are embedded into the binary
//! at compile time by `build.rs` using `minijinja-embed`.

use std::collections::HashMap;

use minijinja::{Environment, Value};

/// A single property of an inline object type (`type.jinja`).
#[derive(Clone, Debug)]
pub struct TypeProperty<'a> {
    pub name: &'a str,
    pub ty: &'a str,
    pub description: Option<&'a str>,
}

/// Data for `function.jinja`.
#[derive(Clone, Debug)]
pub struct FunctionView<'a> {
    pub id: &'a str,
    pub params: Option<&'a [String]>,
    pub response_type_map: &'a [(String, String)],
    pub quote: &'a str,
    pub url: &'a str,
    pub query: &'a str,
    pub method: &'a str,
    pub body_type: Option<&'a str>,
    pub is_not_empty: bool,
}

/// Builds a MiniJinja environment with the embedded templates loaded.
fn environment() -> Environment<'static> {
    let mut env = Environment::new();
    minijinja_embed::load_templates!(&mut env);
    env
}

/// Renders `function.jinja`.
pub fn render_function(view: &FunctionView<'_>) -> String {
    let env = environment();
    let template = env
        .get_template("function.jinja")
        .expect("function.jinja is embedded");

    let body_type_str = view.body_type.unwrap_or("undefined");
    let body_argument = if view.is_not_empty && !body_type_str.is_empty() {
        "body"
    } else {
        "undefined"
    };
    let params: Vec<String> = view.params.map(<[String]>::to_vec).unwrap_or_default();
    let response_type_map: Vec<(String, String)> = view.response_type_map.to_vec();

    template
        .render(minijinja::context! {
            id => view.id,
            params => params,
            response_type_map => response_type_map,
            quote => view.quote,
            url => view.url,
            query => view.query,
            method => view.method,
            body_type_str => body_type_str,
            body_argument => body_argument,
            is_not_empty => view.is_not_empty,
        })
        .expect("function.jinja renders")
}

/// Renders `type.jinja`.
pub fn render_type(properties: &[TypeProperty<'_>]) -> String {
    let env = environment();
    let template = env
        .get_template("type.jinja")
        .expect("type.jinja is embedded");

    let properties: Vec<Value> = properties
        .iter()
        .map(|property| {
            let mut object = HashMap::new();
            object.insert("name", Value::from(property.name));
            object.insert("ty", Value::from(property.ty));
            if let Some(description) = property.description {
                object.insert("description", Value::from(description));
            }
            Value::from_object(object)
        })
        .collect();

    template
        .render(minijinja::context!(properties => properties))
        .expect("type.jinja renders")
}

/// Renders `paths-file.jinja`.
pub fn render_paths_file(key: &str, type_imports: &[String], paths: &[String]) -> String {
    let env = environment();
    let template = env
        .get_template("paths-file.jinja")
        .expect("paths-file.jinja is embedded");

    template
        .render(minijinja::context! {
            key => key,
            type_imports => type_imports.to_vec(),
            paths => paths.to_vec(),
        })
        .expect("paths-file.jinja renders")
}

#[cfg(test)]
mod tests;
