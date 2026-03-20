#!/usr/bin/env bash

set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/release-version.sh [patch|minor|major] [--dry-run]

Defaults:
  bump kind: patch

Behavior:
  1. exits if the worktree is not clean
  2. runs the same checks as .github/workflows/ci.yml
  3. bumps the version in Cargo.toml and Cargo.lock
  4. commits the version bump
  5. creates a matching git tag
  6. pushes the branch and tag to origin
EOF
}

log() {
  printf '==> %s\n' "$*"
}

die() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

run() {
  if [[ "${DRY_RUN}" == "1" ]]; then
    printf '[dry-run] %s\n' "$*"
    return 0
  fi

  "$@"
}

require_clean_worktree() {
  git diff --quiet || die "working tree has unstaged changes"
  git diff --cached --quiet || die "working tree has staged changes"

  if [[ -n "$(git ls-files --others --exclude-standard)" ]]; then
    die "working tree has untracked files"
  fi
}

read_version() {
  perl -ne 'print "$1\n" if /^version = "([0-9]+\.[0-9]+\.[0-9]+)"$/' Cargo.toml | head -n 1
}

bump_version() {
  local current="$1"
  local kind="$2"
  local major minor patch

  IFS='.' read -r major minor patch <<<"$current"

  case "$kind" in
    patch)
      patch=$((patch + 1))
      ;;
    minor)
      minor=$((minor + 1))
      patch=0
      ;;
    major)
      major=$((major + 1))
      minor=0
      patch=0
      ;;
    *)
      die "unsupported bump kind: $kind"
      ;;
  esac

  printf '%s.%s.%s\n' "$major" "$minor" "$patch"
}

update_version_files() {
  local new_version="$1"

  if [[ "${DRY_RUN}" == "1" ]]; then
    printf '[dry-run] update Cargo.toml version to %s\n' "$new_version"
    printf '[dry-run] update Cargo.lock package version to %s\n' "$new_version"
    return 0
  fi

  perl -0pi -e 's/^version = "\K\d+\.\d+\.\d+(?=")/'"$new_version"'/m' Cargo.toml
  perl -0pi -e 's/(name = "git-diff-stat"\nversion = ")\d+\.\d+\.\d+(")/${1}'"$new_version"'${2}/' Cargo.lock
}

main() {
  local bump_kind="patch"
  local branch current_version new_version tag

  DRY_RUN=0

  while (($# > 0)); do
    case "$1" in
      patch|minor|major)
        bump_kind="$1"
        ;;
      --dry-run)
        DRY_RUN=1
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        usage
        die "unknown argument: $1"
        ;;
    esac
    shift
  done

  branch="$(git branch --show-current)"
  [[ -n "$branch" ]] || die "detached HEAD is not supported"

  git remote get-url origin >/dev/null 2>&1 || die "origin remote is not configured"

  require_clean_worktree

  log "running CI-equivalent checks"
  run cargo fmt --check
  run cargo clippy --all-targets --all-features -- -D warnings
  run cargo test -v

  current_version="$(read_version)"
  [[ -n "$current_version" ]] || die "failed to read version from Cargo.toml"

  new_version="$(bump_version "$current_version" "$bump_kind")"
  tag="v${new_version}"

  if git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
    die "tag ${tag} already exists"
  fi

  log "bumping version ${current_version} -> ${new_version}"
  update_version_files "$new_version"

  log "committing release ${tag}"
  run git add Cargo.toml Cargo.lock
  run git commit -m "chore: release ${tag}"

  log "creating tag ${tag}"
  run git tag "${tag}"

  log "pushing ${branch} and ${tag} to origin"
  run git push origin "${branch}"
  run git push origin "${tag}"
}

main "$@"
