# Stats Header Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Always print a descriptive header line above the diff stat output that explains the comparison scope, language filter, and test-filter mode.

**Architecture:** Build a structured description from CLI state and resolved revision information in `main`, then pass that description into the render layer so wording remains centralized. Keep the existing stat rows and summary row intact, only prepending one new line.

**Tech Stack:** Rust, clap, assert_cmd, predicates, cargo test

---

### Task 1: Lock down header output in smoke tests

**Files:**
- Modify: `tests/cli_smoke.rs`

**Step 1: Write the failing test**

Add one assertion for the default working-tree output header and one assertion for `--last --no-test-filter`. Add one additional case for an explicit revision range plus multi-language filter.

**Step 2: Run test to verify it fails**

Run: `cargo test --test cli_smoke -v`
Expected: FAIL because no descriptive header is printed yet.

**Step 3: Write minimal implementation**

Introduce render context and generate the three header dimensions from CLI state.

**Step 4: Run test to verify it passes**

Run: `cargo test --test cli_smoke -v`
Expected: PASS

### Task 2: Add structured header generation

**Files:**
- Modify: `src/main.rs`
- Modify: `src/render.rs`
- Modify: `src/revision.rs`

**Step 1: Write the failing test**

Add a focused unit test for revision-description formatting if needed.

**Step 2: Run test to verify it fails**

Run: `cargo test --test revision -v`
Expected: FAIL if the description helper is missing.

**Step 3: Write minimal implementation**

Create a small description struct or equivalent values, map working tree / last commit / commit patch / revision ranges to human-readable Chinese phrases, and prepend the rendered header line.

**Step 4: Run test to verify it passes**

Run: `cargo test --test revision --test cli_smoke -v`
Expected: PASS

### Task 3: Align README examples

**Files:**
- Modify: `README.md`

**Step 1: Write the failing test**

If needed, extend an existing README presence check or help-output expectation.

**Step 2: Run test to verify it fails**

Run: `cargo test --test readme_presence -v`
Expected: PASS or no change needed.

**Step 3: Write minimal implementation**

Document that the command now prints a header describing revision scope, languages, and test scope.

**Step 4: Run test to verify it passes**

Run: `cargo test --test readme_presence -v`
Expected: PASS
