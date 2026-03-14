use git_diff_stat::revision::RevisionSelection;

#[test]
fn maps_commit_flag_to_single_commit_patch() {
    let selection = RevisionSelection::from_commit("abc123").unwrap();

    assert_eq!(selection.git_diff_args(), vec!["abc123^!".to_string()]);
}
