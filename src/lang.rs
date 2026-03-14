use std::path::Path;

use crate::change::FileChange;

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

fn normalize_lang(lang: &str) -> &str {
    lang.trim()
}

fn detect_language(path: &str) -> Option<&'static str> {
    match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("rs") => Some("rs"),
        Some("js") => Some("js"),
        Some("ts") => Some("ts"),
        _ => None,
    }
}
