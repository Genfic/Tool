fn main() {
    // Embed only the Jinja templates; `typed-fetch.ts` is a static asset loaded
    // separately with `include_str!`.
    minijinja_embed::embed_templates!("src/tools/generate_paths/templates", &[".jinja"]);
}
