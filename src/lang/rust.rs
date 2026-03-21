use std::path::Path;

use crate::filter::{
    RustTestSplit, collect_rust_whole_test_paths, split_file_patch_for_rust_tests,
    split_untracked_rust_source,
};
use crate::patch::FilePatch;

pub fn matches_path(path: &str) -> bool {
    Path::new(path).extension().and_then(|ext| ext.to_str()) == Some("rs")
}

pub fn collect_whole_test_paths(
    sources: &[(String, String)],
) -> Result<std::collections::HashSet<String>, String> {
    collect_rust_whole_test_paths(sources)
}

pub fn split_file_patch(
    patch: &FilePatch,
    old_source: &str,
    new_source: &str,
) -> Result<RustTestSplit, String> {
    split_file_patch_for_rust_tests(patch, old_source, new_source)
}

pub fn split_untracked_source(source: &str) -> Result<RustTestSplit, String> {
    split_untracked_rust_source(source)
}
