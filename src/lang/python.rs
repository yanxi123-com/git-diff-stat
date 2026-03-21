use std::path::Path;

use crate::patch::FilePatch;
use crate::python_tests::{
    PythonTestSplit, collect_python_whole_test_paths, split_file_patch_for_python_tests,
    split_untracked_python_source,
};

pub fn matches_path(path: &str) -> bool {
    Path::new(path).extension().and_then(|ext| ext.to_str()) == Some("py")
}

pub fn collect_whole_test_paths(
    sources: &[(String, String)],
) -> Result<std::collections::HashSet<String>, String> {
    collect_python_whole_test_paths(sources)
}

pub fn split_file_patch(
    patch: &FilePatch,
    old_source: &str,
    new_source: &str,
) -> Result<PythonTestSplit, String> {
    split_file_patch_for_python_tests(patch, old_source, new_source)
}

pub fn split_untracked_source(source: &str) -> Result<PythonTestSplit, String> {
    split_untracked_python_source(source)
}
