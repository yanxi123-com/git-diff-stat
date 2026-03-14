use git_diff_stat::revision::RevisionSelection;

#[test]
fn maps_commit_flag_to_single_commit_patch() {
    let selection = RevisionSelection::from_commit("abc123").unwrap();

    assert_eq!(selection.git_diff_args(), vec!["abc123^!".to_string()]);
}

#[test]
fn maps_single_positional_revision_to_commit_and_head() {
    let selection = RevisionSelection::Revisions(vec!["abc123".to_string()]);

    assert_eq!(
        selection.git_diff_args(),
        vec!["abc123".to_string(), "HEAD".to_string()]
    );
}
