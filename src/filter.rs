use crate::change::line_count;
use crate::patch::{FilePatch, LineKind};
use crate::rust_tests::detect_test_regions;
use std::path::Path;

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
