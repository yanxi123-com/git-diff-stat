#[test]
fn readme_mentions_github_release_install() {
    let readme = std::fs::read_to_string("README.md").unwrap();

    assert!(readme.contains("GitHub Releases"));
    assert!(readme.contains("v0.1.0"));
}
