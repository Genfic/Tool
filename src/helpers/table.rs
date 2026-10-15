//! Port of `helpers/table.ts`.
//!
//! `render` returns the rendered table; the caller writes it with a single
//! `println!`, which produces the same bytes as the original's per-line
//! `console.log` calls.

use crate::helpers::chars as ch;
use crate::helpers::js::start_case;

#[derive(Default)]
pub struct Table {
    header: Option<Vec<String>>,
    body: Vec<Vec<String>>,
    footer: Vec<Vec<String>>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            header: None,
            body: vec![Vec::new()],
            footer: vec![Vec::new()],
        }
    }

    pub fn header(mut self, header: Vec<String>) -> Self {
        self.header = Some(header);
        self
    }

    pub fn body(mut self, body: Vec<Vec<String>>) -> Self {
        self.body = body;
        self
    }

    /// Port of `objectBody`: the header is the key set of the first object and
    /// the rows are the matching values.
    pub fn object_body(mut self, header: Vec<String>, rows: Vec<Vec<String>>) -> Self {
        self.header = Some(header);
        self.body = rows;
        self
    }

    pub fn footer(mut self, footer: Vec<Vec<String>>) -> Self {
        self.footer = footer;
        self
    }

    /// Mirrors `Table#toString` (only kept for completeness; the tools use
    /// [`Table::render`]).
    #[allow(dead_code)]
    pub fn to_flat_string(&self) -> String {
        let mut rows: Vec<Vec<String>> = Vec::new();
        if let Some(header) = &self.header {
            rows.push(header.clone());
        }
        rows.extend(self.body.iter().cloned());
        rows.extend(self.footer.iter().cloned());
        rows.iter()
            .map(|row| row.join(","))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Mirrors `Table#render` and returns the rendered text.
    pub fn render(&self) -> String {
        self.render_with_color(!no_color())
    }

    fn render_with_color(&self, color: bool) -> String {
        let mut all_rows: Vec<Vec<String>> = Vec::new();
        match &self.header {
            Some(header) => all_rows.push(header.clone()),
            None => all_rows.push(Vec::new()),
        }
        all_rows.extend(self.footer.iter().cloned());
        all_rows.extend(self.body.iter().cloned());

        let cols = all_rows.iter().map(Vec::len).max().unwrap_or(0);
        let mut col_widths = vec![0usize; cols];
        for row in &all_rows {
            for (index, cell) in row.iter().enumerate() {
                col_widths[index] = col_widths[index].max(cell.chars().count());
            }
        }

        let mut lines: Vec<String> = Vec::new();
        lines.push(build_horizontal_border(&col_widths, "top"));

        if let Some(header) = &self.header {
            let emphasised: Vec<String> = header.iter().map(|cell| start_case(cell)).collect();
            lines.push(build_row(&emphasised, &col_widths, color));
            lines.push(build_horizontal_border(&col_widths, "middle"));
        }

        for row in &self.body {
            lines.push(build_row(row, &col_widths, false));
        }

        if !self.footer.is_empty() {
            lines.push(build_horizontal_border(&col_widths, "middle"));
            for row in &self.footer {
                lines.push(build_row(row, &col_widths, false));
            }
        }

        lines.push(build_horizontal_border(&col_widths, "bottom"));
        lines.join("\n")
    }
}

fn build_row(row: &[String], col_widths: &[usize], emphasis: bool) -> String {
    let mut cells = Vec::with_capacity(col_widths.len());
    for (index, width) in col_widths.iter().enumerate() {
        let raw = row.get(index).map(String::as_str).unwrap_or("");
        if emphasis {
            let bolded = bold(raw);
            // Pad by the visible width, leaving room for the escape codes.
            let padding = width + (bolded.chars().count() - raw.chars().count());
            cells.push(format!("{bolded:<padding$}"));
        } else {
            cells.push(format!("{raw:<width$}"));
        }
    }
    format!(
        "{} {} {}",
        ch::V,
        cells.join(&format!(" {} ", ch::V)),
        ch::V
    )
}

fn build_horizontal_border(col_widths: &[usize], position: &str) -> String {
    let border: Vec<String> = col_widths
        .iter()
        .map(|width| ch::H.repeat(*width))
        .collect();
    let separator = |middle: &str| format!("{}{}{}", ch::H, middle, ch::H);
    match position {
        "top" => format!(
            "{}{}{}{}{}",
            ch::TL,
            ch::H,
            border.join(&separator(ch::TM)),
            ch::H,
            ch::TR
        ),
        "middle" => format!(
            "{}{}{}{}{}",
            ch::ML,
            ch::H,
            border.join(&separator(ch::MM)),
            ch::H,
            ch::MR
        ),
        _ => format!(
            "{}{}{}{}{}",
            ch::BL,
            ch::H,
            border.join(&separator(ch::BM)),
            ch::H,
            ch::BR
        ),
    }
}

fn bold(text: &str) -> String {
    format!("\u{1b}[1m{text}\u{1b}[0m")
}

/// `Deno.noColor`: the `NO_COLOR` environment variable is set and non-empty.
fn no_color() -> bool {
    std::env::var_os("NO_COLOR").is_some_and(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::Table;

    fn sample() -> Table {
        Table::new()
            .object_body(
                vec!["name".to_string(), "size".to_string()],
                vec![
                    vec!["z.js".to_string(), "5 kB".to_string()],
                    vec!["a.js".to_string(), "3 kB".to_string()],
                ],
            )
            .footer(vec![
                vec!["Total size".to_string(), "8 kB".to_string()],
                vec!["Gzip".to_string(), "2 kB (25.00%)".to_string()],
            ])
    }

    #[test]
    fn renders_like_the_deno_table_without_color() {
        // Column widths are computed across header, body *and* footer rows, so
        // "Total size" (10 chars) widens the first column.
        let expected = [
            "┌────────────┬───────────────┐",
            "│ Name       │ Size          │",
            "├────────────┼───────────────┤",
            "│ z.js       │ 5 kB          │",
            "│ a.js       │ 3 kB          │",
            "├────────────┼───────────────┤",
            "│ Total size │ 8 kB          │",
            "│ Gzip       │ 2 kB (25.00%) │",
            "└────────────┴───────────────┘",
        ]
        .join("\n");
        assert_eq!(sample().render_with_color(false), expected);
    }

    #[test]
    fn bolds_the_header_like_std_fmt_colors() {
        let rendered = sample().render_with_color(true);
        let header = rendered.lines().nth(1).unwrap();
        assert_eq!(
            header,
            "│ \u{1b}[1mName\u{1b}[0m       │ \u{1b}[1mSize\u{1b}[0m          │"
        );
    }
}
