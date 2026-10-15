//! Hand written equivalents of the three Eta templates in
//! `tools/generate-paths/templates`.
//!
//! Eta is configured with `autoTrim: [false, "nl"]` by default, which strips
//! one newline after every closing tag (and the templates themselves are CRLF,
//! with Eta normalising emitted text to LF). The functions below reproduce the
//! exact resulting bytes, including the leading newline of a rendered function
//! and the absence of a trailing newline.

/// Input for [`render_function`], mirroring the `function.eta` view model.
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

/// A single property of an inline object type (`type.eta`).
pub struct TypeProperty<'a> {
    pub name: &'a str,
    pub ty: &'a str,
    pub description: Option<&'a str>,
}

/// Renders `function.eta`.
pub fn render_function(view: &FunctionView<'_>) -> String {
    let body = match (view.body_type, view.is_not_empty) {
        (Some(body_type), true) => format!("body: {body_type}, "),
        _ => String::new(),
    };
    let params = view.params.map(|p| p.join(", ")).unwrap_or_default();
    let trailing = match view.params {
        Some(p) if !p.is_empty() => ", ",
        _ => "",
    };
    let responses = view
        .response_type_map
        .iter()
        .map(|(status, ty)| format!("{status}: {ty}"))
        .collect::<Vec<_>>()
        .join("; ");
    let body_type = view.body_type.unwrap_or("undefined");
    let body_argument = if view.is_not_empty && !body_type.is_empty() {
        "body"
    } else {
        "undefined"
    };
    let method = view.method.to_uppercase();
    let quote = view.quote;
    let url = view.url;
    let query = view.query;

    format!(
        "\nexport const {id} = async ({body}{params}{trailing}headers?: HeadersInit, options?: RequestInit) => await typedFetch<{{ {responses} }}, {body_type}>({quote}{url}{query}{quote},\n    {method},\n    {body_argument},\n    headers,\n    options,\n);",
        id = view.id,
    )
}

/// Renders `type.eta`.
pub fn render_type(properties: &[TypeProperty<'_>]) -> String {
    let mut out = String::from("{\n");
    for property in properties {
        if let Some(description) = property.description
            && !description.is_empty()
        {
            out.push_str(&format!("    /** {description} */\n"));
        }
        out.push_str(&format!("    {}: {};\n", property.name, property.ty));
    }
    out.push('}');
    out
}

/// Renders `paths-file.eta`.
pub fn render_paths_file(key: &str, type_imports: &[String], paths: &[String]) -> String {
    let mut out = String::from(
        "import { DELETE, GET, HEAD, PATCH, POST, PUT, typedFetch } from \"./typed-fetch\";\nimport type {\n",
    );
    for imported in type_imports {
        out.push_str(&format!("\t{imported},\n"));
    }
    out.push_str(&format!(
        "}} from './types-{key}';\nconst _enc = <T>(p: T): T extends string ? string : T => (typeof p === 'string' ? encodeURIComponent(p) : p) as any;\n\n"
    ));
    for path in paths {
        out.push_str(path);
        // The `paths-file` template emits one `\n` after every path block.
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests;
