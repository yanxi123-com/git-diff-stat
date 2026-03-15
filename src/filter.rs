use crate::change::line_count;
use crate::patch::{FilePatch, LineKind};
use crate::rust_tests::{CfgTestModuleImport, detect_cfg_test_module_imports, detect_test_regions};
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustTestSplit {
    pub test_added: usize,
    pub test_deleted: usize,
    pub non_test_added: usize,
    pub non_test_deleted: usize,
}

pub fn is_rust_integration_test_path(path: &str) -> bool {
    if Path::new(path).extension().and_then(|ext| ext.to_str()) != Some("rs") {
        return false;
    }

    Path::new(path)
        .components()
        .any(|component| component.as_os_str() == "tests")
}

pub fn collect_rust_whole_test_paths(
    sources: &[(String, String)],
) -> Result<HashSet<String>, String> {
    let existing_paths = sources
        .iter()
        .filter(|(path, _)| path.ends_with(".rs"))
        .map(|(path, _)| normalize_repo_path(Path::new(path)))
        .collect::<HashSet<_>>();
    let mut whole_test_paths = existing_paths
        .iter()
        .filter(|path| is_rust_integration_test_path(path))
        .cloned()
        .collect::<HashSet<_>>();

    for (path, source) in sources {
        if !path.ends_with(".rs") {
            continue;
        }

        for import in detect_cfg_test_module_imports(source)? {
            if let Some(resolved) = resolve_cfg_test_module_path(path, &import, &existing_paths) {
                whole_test_paths.insert(resolved);
            }
        }
    }

    Ok(whole_test_paths)
}

pub fn split_file_patch_for_rust_tests(
    patch: &FilePatch,
    old_source: &str,
    new_source: &str,
) -> Result<RustTestSplit, String> {
    let old_regions = detect_test_regions(old_source)?;
    let new_regions = detect_test_regions(new_source)?;
    let mut split = RustTestSplit {
        test_added: 0,
        test_deleted: 0,
        non_test_added: 0,
        non_test_deleted: 0,
    };

    for event in &patch.line_events {
        match event.kind {
            LineKind::Added => {
                let line = event
                    .new_line
                    .ok_or_else(|| "added line event missing new line".to_string())?;
                if new_regions.contains_line(line) {
                    split.test_added += 1;
                } else {
                    split.non_test_added += 1;
                }
            }
            LineKind::Deleted => {
                let line = event
                    .old_line
                    .ok_or_else(|| "deleted line event missing old line".to_string())?;
                if old_regions.contains_line(line) {
                    split.test_deleted += 1;
                } else {
                    split.non_test_deleted += 1;
                }
            }
        }
    }

    Ok(split)
}

pub fn split_untracked_rust_source(source: &str) -> Result<RustTestSplit, String> {
    let regions = detect_test_regions(source)?;
    let mut split = RustTestSplit {
        test_added: 0,
        test_deleted: 0,
        non_test_added: 0,
        non_test_deleted: 0,
    };

    for line in 1..=line_count(source) {
        if regions.contains_line(line) {
            split.test_added += 1;
        } else {
            split.non_test_added += 1;
        }
    }

    Ok(split)
}

fn resolve_cfg_test_module_path(
    importer_path: &str,
    import: &CfgTestModuleImport,
    existing_paths: &HashSet<String>,
) -> Option<String> {
    let importer_path = Path::new(importer_path);
    let importer_dir = importer_path.parent().unwrap_or_else(|| Path::new(""));

    if let Some(path_attribute) = &import.path_attribute {
        let resolved = normalize_repo_path(&importer_dir.join(path_attribute));
        return existing_paths.contains(&resolved).then_some(resolved);
    }

    let module_root = implicit_module_root(importer_path)?;
    let file_candidate =
        normalize_repo_path(&module_root.join(format!("{}.rs", import.module_name)));
    if existing_paths.contains(&file_candidate) {
        return Some(file_candidate);
    }

    let mod_candidate = normalize_repo_path(&module_root.join(&import.module_name).join("mod.rs"));
    existing_paths
        .contains(&mod_candidate)
        .then_some(mod_candidate)
}

fn implicit_module_root(importer_path: &Path) -> Option<PathBuf> {
    let importer_dir = importer_path.parent().unwrap_or_else(|| Path::new(""));
    match importer_path.file_name().and_then(|name| name.to_str()) {
        Some("mod.rs") => Some(importer_dir.to_path_buf()),
        Some(_) => Some(importer_dir.join(importer_path.file_stem()?)),
        None => None,
    }
}

fn normalize_repo_path(path: &Path) -> String {
    let mut normalized = Vec::new();

    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value.to_string_lossy().into_owned()),
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            Component::RootDir | Component::Prefix(_) => {}
        }
    }

    normalized.join("/")
}
