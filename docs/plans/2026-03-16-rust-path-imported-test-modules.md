# Rust Path-Imported Test Modules Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `--test` and `--no-test` classify Rust files imported by `#[cfg(test)]` module declarations as whole-file test code.

**Architecture:** Extend the Rust test filter with a whole-file classifier that combines the existing `tests/` path rule with a new repository-aware index of `#[cfg(test)]` module imports. Build the index from relevant Rust sources before stats rendering, then reuse it for tracked and untracked files while leaving AST-based partial splitting unchanged for all other Rust files.

**Tech Stack:** Rust, existing git/diff helpers, `tree-sitter-rust`, `assert_cmd`, temporary git repositories.

---

### Task 1: Add failing regression tests for path-imported `tests.rs`

**Files:**
- Modify: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_filter_counts_cfg_test_path_module_files_as_test() {
    // create src/app.rs with #[cfg(test)] #[path = "app/tests.rs"] mod tests;
    // create src/app/tests.rs with helper code and one assertion change
}

#[test]
fn no_test_filter_excludes_cfg_test_path_module_files() {
    // same repository shape, assert --no-test reports 0 files changed
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke test_filter_counts_cfg_test_path_module_files_as_test -- --exact`
Expected: FAIL because the imported `tests.rs` file is currently treated as non-test.

Run: `cargo test --test cli_smoke no_test_filter_excludes_cfg_test_path_module_files -- --exact`
Expected: FAIL because `--no-test` currently reports the imported file.

**Step 3: Write minimal implementation**

- add repo-aware detection of `#[cfg(test)]` imported whole-file test modules
- thread that classifier into `build_rust_test_stats`

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke test_filter_counts_cfg_test_path_module_files_as_test no_test_filter_excludes_cfg_test_path_module_files -v`
Expected: PASS

### Task 2: Add focused unit coverage for module-path resolution

**Files:**
- Modify: `tests/test_split.rs`
- Modify: `src/filter.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn classifies_explicit_cfg_test_path_module_files() {
    // assert classifier includes src/app/tests.rs
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test classifies_explicit_cfg_test_path_module_files -v`
Expected: FAIL because no classifier exists for path-imported test modules.

**Step 3: Write minimal implementation**

- parse module importer sources for `#[cfg(test)]` and optional `#[path = "..."]`
- resolve module target paths relative to the importer file
- preserve existing `tests/` path behavior

**Step 4: Run test to verify it passes**

Run: `cargo test classifies_explicit_cfg_test_path_module_files -v`
Expected: PASS

### Task 3: Verify and document the final behavior

**Files:**
- Modify: `README.md`

**Step 1: Update docs**

- document that Rust files imported by `#[cfg(test)]` modules are treated as whole-file test code
- retain the note that ordinary production files still use AST-based region splitting

**Step 2: Run targeted tests**

Run: `cargo test --test cli_smoke --test test_split`
Expected: PASS

**Step 3: Run full test suite**

Run: `cargo test`
Expected: PASS
