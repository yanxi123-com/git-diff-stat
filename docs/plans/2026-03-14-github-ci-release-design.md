# GitHub CI And Release Design

## Goal

Add GitHub-native automation for this public repository so that:

- every push and pull request runs a Rust CI baseline
- pushing a version tag publishes downloadable binaries for macOS, Linux, and Windows

The design should stay lightweight, avoid extra release tooling, and match the current Rust/Cargo workflow.

## Scope

This design adds two GitHub Actions workflows:

- `ci.yml` for validation on normal development activity
- `release.yml` for tagged releases

It also updates repository-facing docs so install and release usage are discoverable from GitHub.

## Approach Options

### Option 1: Native GitHub Actions with Cargo commands

Use standard GitHub Actions jobs, shell commands, and release upload steps.

Pros:

- minimal moving parts
- transparent behavior
- easy to debug
- no extra release tool lock-in

Cons:

- more YAML than a specialized release tool
- manual archive naming and checksum generation

### Option 2: Introduce a release helper such as `cargo-dist`

Pros:

- richer release packaging
- less hand-written workflow logic

Cons:

- extra toolchain and config
- more abstraction than needed for this repository right now

### Option 3: CI only, manual release

Pros:

- smallest initial change

Cons:

- does not satisfy the requirement to automate release publishing

## Recommendation

Use Option 1: native GitHub Actions with direct Cargo commands.

This keeps the repository easy to maintain while still providing full CI and release automation. It also leaves room to migrate to `cargo-dist` later if release needs become more complex.

## Workflow Design

### 1. Continuous Integration

Create `.github/workflows/ci.yml`.

Triggers:

- `push`
- `pull_request`

Recommended initial runtime:

- `ubuntu-latest`

Jobs:

1. `fmt`
   - `cargo fmt --check`
2. `clippy`
   - `cargo clippy --all-targets --all-features -- -D warnings`
3. `test`
   - `cargo test -v`

Reasoning:

- keep PR feedback fast
- avoid multiplying runtime cost during daily development
- rely on release workflow for cross-platform build validation

### 2. Release Publishing

Create `.github/workflows/release.yml`.

Trigger:

- tag push matching `v*`

Release gate:

- before publishing artifacts, run Linux validation:
  - `cargo fmt --check`
  - `cargo clippy --all-targets --all-features -- -D warnings`
  - `cargo test -v`

Build matrix:

- `ubuntu-latest` -> `x86_64-unknown-linux-gnu`
- `macos-13` -> `x86_64-apple-darwin`
- `macos-14` -> `aarch64-apple-darwin`
- `windows-latest` -> `x86_64-pc-windows-msvc`

Packaging:

- Unix targets: `.tar.gz`
- Windows target: `.zip`

Artifact names:

- `git-diff-stat-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`
- `git-diff-stat-vX.Y.Z-x86_64-apple-darwin.tar.gz`
- `git-diff-stat-vX.Y.Z-aarch64-apple-darwin.tar.gz`
- `git-diff-stat-vX.Y.Z-x86_64-pc-windows-msvc.zip`

Also publish:

- `SHA256SUMS`

### 3. Versioning Rules

Use:

- `Cargo.toml` version as the package version
- Git tag format `vX.Y.Z`

The release workflow should derive the release version from the tag and use it in archive names. The process should assume the tag and `Cargo.toml` version are kept in sync by the maintainer.

## Documentation Changes

Update `README.md` to add:

- CI badge placeholder or section
- release download note
- installation from GitHub Releases

Example release install guidance should show users how to:

- download an archive
- extract it
- place `git-diff-stat` on `PATH`

## Error Handling And Tradeoffs

### CI failures

Fail fast and block merge confidence. No retry logic in workflow design beyond GitHub Actions defaults.

### Release failures

If packaging or upload fails on any matrix target, the release workflow should fail rather than publish a partial release silently.

### Secrets

Prefer using the default `GITHUB_TOKEN` where possible. No custom secrets should be required for the initial release workflow.

## Deferred Work

Not included in this round:

- signing artifacts
- Homebrew/Scoop packaging
- automatic changelog generation
- draft releases with notes templating
- `cargo-dist`
- nightly/pre-release channels
