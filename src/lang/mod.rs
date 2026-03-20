use crate::change::FileChange;

const SUPPORTED_LANGS: &[&str] = &["rs", "py", "js", "ts", "jsx", "tsx", "cjs", "mjs"];

pub mod javascript;
pub mod python;
pub mod rust;

pub fn filter_by_langs(changes: &[FileChange], langs: &[&str]) -> Result<Vec<FileChange>, String> {
    let requested = langs
        .iter()
        .map(|lang| normalize_lang(lang))
        .collect::<Vec<_>>();

    Ok(changes
        .iter()
        .filter(|change| {
            detect_language(&change.path)
                .map(|language| requested.contains(&language))
                .unwrap_or(false)
        })
        .cloned()
        .collect())
}

pub fn parse_langs(value: Option<&str>) -> Vec<&str> {
    value
        .map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .collect()
        })
        .unwrap_or_else(|| supported_langs().to_vec())
}

pub fn supported_langs() -> &'static [&'static str] {
    SUPPORTED_LANGS
}

pub fn detect_language(path: &str) -> Option<&'static str> {
    if rust::matches_path(path) {
        Some("rs")
    } else if python::matches_path(path) {
        Some("py")
    } else {
        javascript::detect_language(path)
    }
}

fn normalize_lang(lang: &str) -> &str {
    lang.trim()
}
