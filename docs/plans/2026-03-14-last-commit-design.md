# Last Commit Shortcut Design

## Goal

Add a fast CLI shortcut to inspect the diff stat for the most recent commit by running `git diff-stat --last`.

## User Experience

- `git diff-stat --last` shows the patch introduced by `HEAD`
- Semantics match `git diff HEAD^!`
- `--last` can still be combined with existing filters such as `--lang`, `--test`, and `--no-test`
- `--last` cannot be combined with `--commit` or positional revision arguments

## Recommended Approach

Add a new boolean CLI flag and map it to the existing single-commit patch path by reusing `RevisionSelection::CommitPatch("HEAD")`.

This keeps the feature as a thin sugar layer:

- no new revision model is required
- existing patch endpoint logic continues to produce `HEAD^!` and `HEAD^ -> HEAD`
- documentation and help text stay explicit

## Alternatives Considered

### Extend `--commit` to allow omission

Rejected because `--commit` without a value is less explicit, makes `clap` parsing less clear, and does not match the requested interface.

### Add a positional keyword like `last`

Rejected because it would conflict with valid git revision names and create ambiguous parsing.

## Testing

- CLI parsing test for `--last` conflicts
- revision-selection test proving `--last` maps to `HEAD^!`
- smoke-level integration test proving `git diff-stat --last` reports the latest committed file changes

