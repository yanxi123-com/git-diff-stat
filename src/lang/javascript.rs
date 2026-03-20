use std::collections::HashSet;
use std::path::Path;

const JS_TS_FAMILY_LANGS: &[&str] = &["js", "ts", "jsx", "tsx", "cjs", "mjs"];

pub fn family_langs() -> &'static [&'static str] {
    JS_TS_FAMILY_LANGS
}

pub fn detect_language(path: &str) -> Option<&'static str> {
    match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("js") => Some("js"),
        Some("ts") => Some("ts"),
        Some("jsx") => Some("jsx"),
        Some("tsx") => Some("tsx"),
        Some("cjs") => Some("cjs"),
        Some("mjs") => Some("mjs"),
        _ => None,
    }
}

pub fn collect_whole_test_paths(
    sources: &[(String, String)],
) -> Result<HashSet<String>, String> {
    Ok(sources
        .iter()
        .map(|(path, _)| path)
        .filter(|path| is_whole_test_path(path))
        .cloned()
        .collect())
}

fn is_whole_test_path(path: &str) -> bool {
    let Some(language) = detect_language(path) else {
        return false;
    };

    if Path::new(path).components().any(|component| {
        matches!(
            component.as_os_str().to_str(),
            Some("__tests__" | "e2e" | "cypress" | "playwright")
        )
    }) {
        return true;
    }

    let filename = Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let Some(stem) = filename.strip_suffix(&format!(".{language}")) else {
        return false;
    };

    stem.ends_with(".test") || stem.ends_with(".spec") || stem.ends_with(".cy")
}
