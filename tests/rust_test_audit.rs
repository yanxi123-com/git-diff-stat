use std::fs;
use std::path::PathBuf;

use assert_cmd::Command;
use git_diff_stat::audit::{AuditConfig, AuditDecision, scan_paths};
use predicates::prelude::predicate;
use tempfile::tempdir;

#[test]
fn flags_heavy_inline_test_modules_for_extraction() {
    let tempdir = tempdir().unwrap();
    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        heavy_inline_test_source("prod", 48),
    )
    .unwrap();

    let report = scan_paths(
        tempdir.path(),
        &[PathBuf::from("src")],
        &AuditConfig::default(),
    )
    .unwrap();

    assert_eq!(report.findings.len(), 1);
    let finding = &report.findings[0];
    assert_eq!(finding.path, "src/lib.rs");
    assert_eq!(finding.decision, AuditDecision::ExtractNow);
    assert!(finding.test_lines >= 120);
}

#[test]
fn skips_files_that_only_import_external_test_modules() {
    let tempdir = tempdir().unwrap();
    fs::create_dir_all(tempdir.path().join("src/runtime")).unwrap();
    fs::write(
        tempdir.path().join("src/runtime.rs"),
        "#[cfg(test)]\n#[path = \"runtime/tests.rs\"]\nmod tests;\n\nfn prod() {}\n",
    )
    .unwrap();
    fs::write(
        tempdir.path().join("src/runtime/tests.rs"),
        "#[test]\nfn covers_prod() {\n    assert_eq!(2 + 2, 4);\n}\n",
    )
    .unwrap();

    let report = scan_paths(
        tempdir.path(),
        &[PathBuf::from("src")],
        &AuditConfig::default(),
    )
    .unwrap();

    assert!(report.findings.is_empty());
}

#[test]
fn cli_scans_current_directory_by_default() {
    let tempdir = tempdir().unwrap();
    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("src/lib.rs"),
        heavy_inline_test_source("prod", 48),
    )
    .unwrap();

    Command::cargo_bin("rust-test-audit")
        .unwrap()
        .current_dir(tempdir.path())
        .args(["--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"path\": \"src/lib.rs\""));
}

fn heavy_inline_test_source(prod_name: &str, test_count: usize) -> String {
    let mut source = format!("fn {prod_name}() {{}}\n\n#[cfg(test)]\nmod tests {{\n");
    for index in 0..test_count {
        source.push_str(&format!(
            "    #[test]\n    fn case_{index}() {{\n        assert_eq!({index}, {index});\n    }}\n\n"
        ));
    }
    source.push_str("}\n");
    source
}
