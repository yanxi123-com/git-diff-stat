# git-diff-stat

`git-diff-stat` is a Rust CLI that extends `git diff --stat` with:

- untracked files included in default stats
- language filtering with `--lang`
- Rust test-only and non-test-only stats with `--test` and `--no-test`
- single-commit and revision-range support

## Install

### Local development build

```bash
cargo install --path .
```

### Install from a release artifact

Copy the compiled `git-diff-stat` binary into any directory on your `PATH`, for example:

```bash
cp target/release/git-diff-stat ~/.local/bin/
```

## Git integration

Git automatically treats an executable named `git-diff-stat` as the `git diff-stat` subcommand. Once the binary is on your `PATH`, you can run:

```bash
git diff-stat
git diff-stat --commit HEAD
git diff-stat HEAD~1..HEAD --lang rs
git diff-stat --lang rs --test
```

## Usage

```bash
git diff-stat [<rev> | <rev1> <rev2> | <rev-range>] [--lang rs,js] [--test | --no-test]
```

### Revision forms

- `git diff-stat`
- `git diff-stat --commit <commit>`
- `git diff-stat <commit>`
- `git diff-stat <a> <b>`
- `git diff-stat <a>..<b>`
- `git diff-stat <a>...<b>`

`<commit>` by itself is treated as `<commit> HEAD`, so it reports the diff from that commit to `HEAD`.

## Notes

- `--lang` currently uses file extensions.
- `--test` and `--no-test` currently apply Rust-aware code-region splitting for `#[cfg(test)]` modules and test-annotated functions such as `#[test]` and `#[tokio::test]`.
- Output is intentionally close to `git diff --stat`, but not byte-for-byte identical.
