use std::path::Path;

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
