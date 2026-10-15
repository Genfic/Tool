//! Port of `helpers/stringUtils.ts`.

/// Removes every leading and trailing occurrence of `trim` from `source`.
///
/// Mirrors the hand rolled `trim` helper in `helpers/stringUtils.ts`, which
/// only ever strips a single character (not a set of characters).
pub fn trim(source: &str, trim: char) -> String {
    source.trim_matches(trim).to_string()
}

/// `trim(trim(dest, "/"), "\\")` — the destination normalisation shared by
/// `generate-imports` and `generate-preloads`.
pub fn trim_destination(dest: &str) -> String {
    trim(&trim(dest, '/'), '\\')
}

#[cfg(test)]
mod tests {
    use super::trim;

    #[test]
    fn strips_single_character_from_both_ends() {
        assert_eq!(trim("///fonts///", '/'), "fonts");
        assert_eq!(trim("\\\\fonts\\\\", '\\'), "fonts");
        assert_eq!(trim("fonts", '/'), "fonts");
        assert_eq!(trim("", '/'), "");
        assert_eq!(trim("/", '/'), "");
    }
}
