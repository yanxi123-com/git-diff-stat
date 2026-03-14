# GitHub CI And Release Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add GitHub Actions workflows for Rust CI and multi-platform tagged releases, plus the related README updates for public distribution.

**Architecture:** The repository will use two native GitHub Actions workflows: a lightweight CI workflow for push and pull request validation on Linux, and a release workflow that first re-validates on Linux and then builds release archives for macOS, Linux, and Windows. README guidance will explain how to consume tagged binaries from GitHub Releases.

**Tech Stack:** GitHub Actions, Cargo, shell packaging commands, `tar`, PowerShell `Compress-Archive`, existing Rust test suite.

---

### Task 1: Add help and README assertions for release guidance

**Files:**
- Modify: `tests/cli_smoke.rs`
- Create: `tests/readme_presence.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn readme_mentions_github_release_install() {
    let readme = std::fs::read_to_string("README.md").unwrap();
    assert!(readme.contains("GitHub Releases"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test readme_presence readme_mentions_github_release_install -v`
Expected: FAIL if release install guidance is missing.

**Step 3: Write minimal implementation**

- add or refine README text so GitHub Releases installation is documented
- keep help examples aligned with release-facing usage

**Step 4: Run test to verify it passes**

Run: `cargo test --test readme_presence --test cli_smoke -v`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md tests/readme_presence.rs tests/cli_smoke.rs
git commit -m "docs: align release install guidance"
```

### Task 2: Add the CI workflow

**Files:**
- Create: `.github/workflows/ci.yml`

**Step 1: Write the failing test**

Create a shell-level verification step in this task rather than a Rust test:

Run: `test -f .github/workflows/ci.yml`
Expected: FAIL because the workflow does not exist yet.

**Step 2: Run verification to confirm it fails**

Run: `test -f .github/workflows/ci.yml`
Expected: exit code 1

**Step 3: Write minimal implementation**

- create a workflow triggered by `push` and `pull_request`
- run:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test -v`

**Step 4: Run verification to confirm it passes**

Run: `test -f .github/workflows/ci.yml && sed -n '1,220p' .github/workflows/ci.yml`
Expected: file exists and contains the expected commands

**Step 5: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: add github actions validation workflow"
```

### Task 3: Add the release workflow skeleton

**Files:**
- Create: `.github/workflows/release.yml`

**Step 1: Write the failing test**

Run: `test -f .github/workflows/release.yml`
Expected: FAIL because the workflow does not exist yet.

**Step 2: Run verification to confirm it fails**

Run: `test -f .github/workflows/release.yml`
Expected: exit code 1

**Step 3: Write minimal implementation**

- trigger on tags matching `v*`
- add a Linux validation job running:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test -v`
- add a matrix build job for:
  - Linux x86_64
  - macOS x86_64
  - macOS arm64
  - Windows x86_64

**Step 4: Run verification to confirm it passes**

Run: `sed -n '1,260p' .github/workflows/release.yml`
Expected: workflow exists with tag trigger and validation job

**Step 5: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add tagged release workflow skeleton"
```

### Task 4: Add packaging and checksum generation

**Files:**
- Modify: `.github/workflows/release.yml`

**Step 1: Write the failing test**

Use content verification:

Run: `rg "SHA256SUMS|tar.gz|Compress-Archive" .github/workflows/release.yml`
Expected: FAIL or incomplete output before packaging logic is added.

**Step 2: Run verification to confirm it fails**

Run: `rg "SHA256SUMS|tar.gz|Compress-Archive" .github/workflows/release.yml`
Expected: missing one or more required patterns

**Step 3: Write minimal implementation**

- package Unix binaries into `.tar.gz`
- package Windows binary into `.zip`
- generate `SHA256SUMS`
- upload archives and checksums to the GitHub Release

**Step 4: Run verification to confirm it passes**

Run: `rg "SHA256SUMS|tar.gz|Compress-Archive|gh release upload|actions/upload-artifact" .github/workflows/release.yml`
Expected: all required packaging and upload markers are present

**Step 5: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "ci: add multi-platform release packaging"
```

### Task 5: Document GitHub Releases usage in README

**Files:**
- Modify: `README.md`

**Step 1: Write the failing test**

```rust
#[test]
fn readme_mentions_tagged_releases_and_platform_archives() {
    let readme = std::fs::read_to_string("README.md").unwrap();
    assert!(readme.contains("v0.1.0"));
    assert!(readme.contains("GitHub Releases"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --test readme_presence -v`
Expected: FAIL until release-specific guidance exists.

**Step 3: Write minimal implementation**

- add a `GitHub Releases` section
- describe archive names and install flow
- mention tag format `vX.Y.Z`

**Step 4: Run test to verify it passes**

Run: `cargo test --test readme_presence -v`
Expected: PASS

**Step 5: Commit**

```bash
git add README.md tests/readme_presence.rs
git commit -m "docs: add github releases usage guidance"
```

### Task 6: Verify workflow syntax and repository health

**Files:**
- Modify: `none`

**Step 1: Run the Rust suite**

Run: `cargo test -v`
Expected: PASS

**Step 2: Run formatting**

Run: `cargo fmt --check`
Expected: PASS

**Step 3: Run linting**

Run: `cargo clippy --all-targets --all-features -- -D warnings`
Expected: PASS

**Step 4: Inspect workflow files**

Run:

```bash
sed -n '1,220p' .github/workflows/ci.yml
sed -n '1,320p' .github/workflows/release.yml
```

Expected: workflows contain the intended triggers, commands, matrix, packaging, and upload steps.

**Step 5: Commit**

```bash
git add -A
git commit -m "chore: finalize github ci and release automation"
```
