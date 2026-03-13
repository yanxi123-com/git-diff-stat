# Git Diff Stat Enhanced Design

## Goal

Build a long-term maintainable Rust CLI named `git-diff-stat` that behaves like `git diff --stat`, but also:

- includes untracked files in default stats
- supports language filtering with `--lang`
- supports Rust test-only and non-test-only stats with `--test` and `--no-test`
- supports single-commit, commit-to-HEAD, and native git revision/range inputs

The binary should integrate as a git custom subcommand so users can run `git diff-stat`.

## Product Scope

### Default behavior

Without revision arguments, the tool reports working tree changes and untracked files. Untracked files count as full-file additions from `0 -> current line count`.

### Revision behavior

The CLI accepts native git diff-style revision inputs:

- `git diff-stat`
- `git diff-stat A B`
- `git diff-stat A..B`
- `git diff-stat A...B`

It also adds a sugar flag for single-commit patch inspection:

- `git diff-stat --commit <commit>`

`--commit <commit>` is defined as the patch introduced by that commit, equivalent to `<commit>^!`.

### Language filtering

`--lang` accepts a comma-separated list such as `--lang rs` or `--lang js,ts`. Filtering is file-extension based in v1.

### Rust test filtering

`--test` and `--no-test` are core features in v1 and are not reduced to file-path filtering. The tool must split stats inside a single Rust file so test-only changes and non-test changes can be reported separately.

V1 Rust test code includes:

- `#[cfg(test)] mod ... { ... }`
- functions annotated with test attributes such as `#[test]` and `#[tokio::test]`

V1 does not include doctest extraction.

## Recommended Architecture

The implementation should use Rust and shell out to `git` for repository truth, while keeping all product logic in-process.

### 1. CLI layer

Use `clap` to parse:

- revision inputs
- `--commit`
- `--lang`
- `--test`
- `--no-test`

Validation rules:

- `--test` and `--no-test` are mutually exclusive
- `--commit` cannot be combined with positional revisions
- `--test` and `--no-test` only support Rust files in v1

### 2. Git adapter

Use `std::process::Command` to call git commands such as:

- `git diff --numstat -z`
- `git diff --name-status -z`
- `git diff --unified=0 --no-ext-diff --find-renames`
- `git ls-files --others --exclude-standard`
- `git show <rev>:<path>` when old content is needed

Git remains the source of diff truth. The program owns classification, filtering, and rendering.

### 3. Change model

Normalize all changes into internal structs such as:

- revision selection
- file change metadata
- old/new file content when needed
- per-file added/deleted counts
- optional split counts for test vs non-test

This normalized layer prevents git invocation details from leaking into rendering or filtering logic.

### 4. Language classifier

Use extension mapping in v1:

- `rs`
- `js`
- `ts`

Unknown extensions stay unclassified and are excluded when `--lang` is set.

### 5. Rust test-region detector

Use `tree-sitter` with `tree-sitter-rust` to parse both old and new Rust source files and record line ranges for:

- modules annotated with `#[cfg(test)]`
- functions annotated with test attributes

The detector returns sets of line intervals for old and new file versions. Added lines are classified against new-file test intervals. Deleted lines are classified against old-file test intervals.

### 6. Patch mapper

Parse unified diffs at hunk level. For each `+` or `-` line:

- map to the relevant file line number
- decide whether the line belongs to a Rust test region or non-test region
- accumulate counts separately

This allows a single `.rs` file to contribute partially to `--test` and partially to `--no-test`.

### 7. Renderer

Render output close to `git diff --stat`:

- one file line per included file
- total changed count
- insertions and deletions summary

In `--test` or `--no-test` mode, file rows may display partial counts for a file.

## Error Handling

Three error classes should be explicit:

1. Git usage errors
   - invalid revisions
   - not in a repository
   - failed content lookup

2. CLI contract errors
   - incompatible flags
   - unsupported filter combinations

3. Core analysis errors
   - failed Rust parsing for files that require test splitting
   - malformed patch input

For v1, Rust parsing failures in test-filter mode should fail fast instead of silently degrading, because silent fallback would make the core feature untrustworthy.

## Testing Strategy

### Unit tests

Cover:

- CLI parsing and validation
- revision resolution
- language selection
- unified diff hunk parsing
- Rust test-region extraction
- line-to-region mapping

### Integration tests

Use temporary git repositories to validate:

- default mode includes untracked files
- `--lang` filtering
- `--commit <rev>`
- `A..B` and `A...B`
- mixed production/test edits in a single Rust file
- `tests/` files plus inline `#[cfg(test)]` edits

### Golden-style assertions

Add representative output assertions for human-readable stat rendering without requiring byte-for-byte parity with git.

## Why Rust

Rust is the best fit because the tool is intended for long-term maintenance and distribution:

- produces a single portable binary
- has strong CLI and parsing libraries
- supports precise, typed modeling of revision and diff data
- can evolve from extension-based filtering to richer language analysis without changing the product boundary

## Deferred Work

Not required in v1:

- doctest support
- exact visual parity with git's stat graph width heuristics
- non-Rust semantic test detection
- JSON output
- libgit2 migration
