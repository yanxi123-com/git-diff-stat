# git-diff-stat

`git-diff-stat` is a Rust CLI that extends `git diff --stat` with:

- untracked files included in default stats
- language filtering with `--lang`
- Rust-only, non-test-only stats by default, with `--test`, `--no-test`, and `--no-test-filter`
- single-commit and revision-range support

This repository also ships `rust-test-audit`, a companion CLI for auditing Rust source trees
and flagging files where inline test code has grown large enough to consider extracting into
separate `tests.rs` modules.

## Install

### Local development build

```bash
cargo install --path .
```

This installs both `git-diff-stat` and `rust-test-audit`.

### Install from a release artifact

Copy the compiled binaries into any directory on your `PATH`, for example:

```bash
cp target/release/git-diff-stat ~/.local/bin/
cp target/release/rust-test-audit ~/.local/bin/
```

### Install from GitHub Releases

Download the archive for your platform from GitHub Releases, extract it, and place the `git-diff-stat` binary on your `PATH`.

## GitHub Releases

Tagged releases use the `vX.Y.Z` format, for example `v0.1.0`.

Release assets are published per platform, for example:

- `git-diff-stat-v0.1.0-x86_64-unknown-linux-gnu.tar.gz`
- `git-diff-stat-v0.1.0-x86_64-apple-darwin.tar.gz`
- `git-diff-stat-v0.1.0-aarch64-apple-darwin.tar.gz`
- `git-diff-stat-v0.1.0-x86_64-pc-windows-msvc.zip`

### Release helper script

This repository includes a helper script for cutting a new version tag:

```bash
./scripts/release-version.sh
```

By default it bumps the patch version, requires a clean worktree, runs the same checks as [ci.yml](./.github/workflows/ci.yml), updates `Cargo.toml` and `Cargo.lock`, then commits, tags, and pushes to `origin`.

Examples:

```bash
./scripts/release-version.sh
./scripts/release-version.sh minor
./scripts/release-version.sh --dry-run
```

## Git integration

Git automatically treats an executable named `git-diff-stat` as the `git diff-stat` subcommand. Once the binary is on your `PATH`, you can run:

```bash
git diff-stat
git diff-stat --commit HEAD
git diff-stat --last
git diff-stat --last --no-test-filter
git diff-stat HEAD~1..HEAD --lang py --no-test-filter
git diff-stat --test
```

## Usage

```bash
git diff-stat [<rev> | <rev1> <rev2> | <rev-range>] [--lang rs,js] [--test | --no-test | --no-test-filter]
```

Defaults:

- `--lang` defaults to `rs`
- test filtering defaults to `--no-test`
- output always begins with a header line describing the comparison scope, languages, and test scope

## Rust Test Audit

```bash
rust-test-audit [--root <dir>] [--path <dir>]... [--format table|json|markdown]
```

Examples:

```bash
rust-test-audit
rust-test-audit --format json
rust-test-audit --root /path/to/repo --path winq-coin/src --format markdown
```

Defaults:

- `--root` defaults to the current directory
- `--path` defaults to the current directory

The audit skips `tests.rs` files and Rust files under `tests/`, then reports files whose inline
test regions cross configurable density thresholds.

### Revision forms

- `git diff-stat`
- `git diff-stat --commit <commit>`
- `git diff-stat --last`
- `git diff-stat <commit>`
- `git diff-stat <a> <b>`
- `git diff-stat <a>..<b>`
- `git diff-stat <a>...<b>`

`<commit>` by itself is treated as `<commit> HEAD`, so it reports the diff from that commit to `HEAD`.

## Notes

- `--lang` currently uses file extensions.
- `--test` and `--no-test` treat Rust files under `tests/` and Rust files imported by `#[cfg(test)]` module declarations as whole-file test code. Other Rust files still use code-region splitting for `#[cfg(test)]` modules and test-annotated functions such as `#[test]` and `#[tokio::test]`.
- `--no-test-filter` disables Rust test splitting entirely and reports full-file stats for the selected languages.
- because `--lang` defaults to `rs`, use `--no-test-filter --lang <langs>` when you want non-Rust output.
- `--last` is sugar for the patch introduced by `HEAD`, equivalent to `HEAD^!`.
- rendered output starts with a Chinese description line such as `未提交的 rs 文件中，非测试代码统计如下：`.
- Output is intentionally close to `git diff --stat`, but not byte-for-byte identical.
