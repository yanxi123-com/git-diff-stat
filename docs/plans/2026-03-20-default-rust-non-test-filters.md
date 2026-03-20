# Default Rust Non-Test Filters Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `git-diff-stat` default to Rust-only, non-test-only output, and add a flag to disable test filtering.

**Architecture:** Keep language filtering and test filtering as independent dimensions. Represent the test-dimension behavior as three states in CLI handling: test-only, non-test-only (default), and unfiltered. Preserve the existing Rust-specific test splitting logic and bypass it entirely when test filtering is disabled.

**Tech Stack:** Rust, clap, assert_cmd, predicates, cargo test

---

### Task 1: Lock down CLI argument semantics

**Files:**
- Modify: `tests/cli_args.rs`

**Step 1: Write the failing test**

Add assertions that `--no-test-filter` conflicts with `--test` and `--no-test`.

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_args -v`
Expected: FAIL because `--no-test-filter` does not exist yet.

**Step 3: Write minimal implementation**

Add the new clap flag and conflict rules.

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_args -v`
Expected: PASS

### Task 2: Lock down default output behavior

**Files:**
- Modify: `tests/cli_smoke.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test**

Add one regression proving default `--last` output excludes a Rust integration test file and a non-Rust file, and one regression proving `--no-test-filter` includes both the Rust source file and Rust test file while keeping default language filtering.

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke -v`
Expected: FAIL because current defaults do not match the new semantics.

**Step 3: Write minimal implementation**

Default `--lang` to `rs`. Compute test filter mode as default non-test, explicit test-only, or explicit unfiltered. Reuse full stats rendering when filtering is disabled.

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke -v`
Expected: PASS

### Task 3: Align help and README

**Files:**
- Modify: `src/cli.rs`
- Modify: `README.md`

**Step 1: Write the failing test**

Adjust help-output expectations to mention the new default-oriented example and flag.

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke help_mentions_common_examples -v`
Expected: FAIL because help text still documents the old usage.

**Step 3: Write minimal implementation**

Update `after_help`, usage text, and README examples so they describe default Rust/non-test behavior and `--no-test-filter`.

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke help_mentions_common_examples -v`
Expected: PASS
