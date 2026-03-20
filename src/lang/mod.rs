use std::path::Path;

use crate::change::FileChange;

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
            detect_language(&change.old_path)
                .into_iter()
                .chain(detect_language(&change.new_path))
                .any(|language| requested.contains(&language))
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
        .unwrap_or_default()
}

pub fn detect_language(path: &str) -> Option<&'static str> {
    if rust::matches_path(path) {
        Some("rs")
    } else if python::matches_path(path) {
        Some("py")
    } else {
        match Path::new(path).extension().and_then(|ext| ext.to_str()) {
            Some("js") => Some("js"),
            Some("ts") => Some("ts"),
            _ => None,
        }
    }
}

fn normalize_lang(lang: &str) -> &str {
    lang.trim()
}
