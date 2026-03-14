use std::fs;
use std::path::Path;
use std::process::Command;

use git_diff_stat::change::collect_working_tree_changes;
use git_diff_stat::git::Git;
use tempfile::tempdir;

#[test]
fn includes_untracked_files_as_added_lines() {
    let tempdir = tempdir().unwrap();
    init_repo(tempdir.path());

    fs::write(tempdir.path().join("tracked.rs"), "fn tracked() {}\n").unwrap();
    run_git(tempdir.path(), ["add", "tracked.rs"]);
    run_git(tempdir.path(), ["commit", "-m", "initial"]);

    fs::write(
        tempdir.path().join("new_file.rs"),
        "fn one() {}\nfn two() {}\n",
    )
    .unwrap();

    let git = Git::new(tempdir.path());
    let changes = collect_working_tree_changes(&git).unwrap();
    let new_file = changes
        .iter()
        .find(|change| change.path == "new_file.rs")
        .unwrap();

    assert_eq!(new_file.added, 2);
    assert_eq!(new_file.deleted, 0);
    assert!(new_file.untracked);
}

fn init_repo(repo: &Path) {
    run_git(repo, ["init"]);
    run_git(repo, ["config", "user.name", "Codex"]);
    run_git(repo, ["config", "user.email", "codex@example.com"]);
}

fn run_git<const N: usize>(repo: &Path, args: [&str; N]) {
    let status = Command::new("git").args(args).current_dir(repo).status().unwrap();
    assert!(status.success());
}
