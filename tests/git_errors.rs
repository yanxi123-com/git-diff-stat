use git_diff_stat::git::Git;
use tempfile::tempdir;

#[test]
fn returns_clear_error_outside_repository() {
    let tempdir = tempdir().unwrap();
    let git = Git::new(tempdir.path());

    let err = git.diff_numstat(&[]).unwrap_err();

    assert!(err.to_string().contains("git repository"));
}
