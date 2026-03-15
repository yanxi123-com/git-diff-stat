use git_diff_stat::filter::{
    collect_rust_whole_test_paths, is_rust_integration_test_path, split_file_patch_for_rust_tests,
};
use git_diff_stat::patch::parse_patch;

const OLD_SOURCE: &str = "\
fn production() { assert_eq!(1, 1); }

#[cfg(test)]
mod tests {
    #[test]
    fn basic() { assert_eq!(2, 2); }
}
";

const NEW_SOURCE: &str = "\
fn production() { assert_eq!(1, 2); }

#[cfg(test)]
mod tests {
    #[test]
    fn basic() { assert_eq!(3, 3); }
}
";

const PATCH: &str = "\
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,6 +1,6 @@
-fn production() { assert_eq!(1, 1); }
+fn production() { assert_eq!(1, 2); }
 
 #[cfg(test)]
 mod tests {
     #[test]
-    fn basic() { assert_eq!(2, 2); }
+    fn basic() { assert_eq!(3, 3); }
 }
";

#[test]
fn counts_test_and_non_test_changes_separately_in_same_file() {
    let patch = parse_patch(PATCH).unwrap();
    let file = &patch.files[0];
    let split = split_file_patch_for_rust_tests(file, OLD_SOURCE, NEW_SOURCE).unwrap();

    assert_eq!(split.non_test_added, 1);
    assert_eq!(split.non_test_deleted, 1);
    assert_eq!(split.test_added, 1);
    assert_eq!(split.test_deleted, 1);
}

#[test]
fn classifies_rust_integration_test_paths_by_tests_segment() {
    assert!(is_rust_integration_test_path("tests/foo.rs"));
    assert!(is_rust_integration_test_path("crates/app/tests/foo.rs"));
    assert!(!is_rust_integration_test_path("src/tests_support/foo.rs"));
    assert!(!is_rust_integration_test_path("src/lib.rs"));
}

#[test]
fn classifies_cfg_test_path_imported_module_files() {
    let sources = vec![
        (
            "src/runtime.rs".to_string(),
            "#[cfg(test)]\n#[path = \"runtime/tests.rs\"]\nmod tests;\n".to_string(),
        ),
        (
            "src/runtime/tests.rs".to_string(),
            "fn helper() {\n    assert_eq!(1, 1);\n}\n".to_string(),
        ),
    ];

    let paths = collect_rust_whole_test_paths(&sources).unwrap();

    assert!(paths.contains("src/runtime/tests.rs"));
    assert!(!paths.contains("src/runtime.rs"));
}

#[test]
fn classifies_cfg_test_implicit_module_files() {
    let sources = vec![
        (
            "src/runtime.rs".to_string(),
            "#[cfg(test)]\nmod tests;\n".to_string(),
        ),
        (
            "src/runtime/tests.rs".to_string(),
            "fn helper() {\n    assert_eq!(1, 1);\n}\n".to_string(),
        ),
    ];

    let paths = collect_rust_whole_test_paths(&sources).unwrap();

    assert!(paths.contains("src/runtime/tests.rs"));
}
