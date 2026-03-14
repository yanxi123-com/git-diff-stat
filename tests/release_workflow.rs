use std::fs;

#[test]
fn release_workflow_uses_supported_intel_macos_runner() {
    let workflow = fs::read_to_string(".github/workflows/release.yml")
        .expect("release workflow should be readable");

    assert!(
        workflow.contains("macos-15-intel"),
        "release workflow should use the supported Intel macOS runner label"
    );
    assert!(
        !workflow.contains("macos-13"),
        "release workflow should not reference the outdated macos-13 label"
    );
}

#[test]
fn workflows_use_node24_ready_github_actions() {
    let ci_workflow =
        fs::read_to_string(".github/workflows/ci.yml").expect("ci workflow should be readable");
    let release_workflow = fs::read_to_string(".github/workflows/release.yml")
        .expect("release workflow should be readable");

    assert!(
        ci_workflow.contains("actions/checkout@v5"),
        "ci workflow should use checkout@v5"
    );
    assert!(
        release_workflow.contains("actions/checkout@v5"),
        "release workflow should use checkout@v5"
    );
    assert!(
        release_workflow.contains("actions/upload-artifact@v6"),
        "release workflow should use upload-artifact@v6"
    );
    assert!(
        release_workflow.contains("actions/download-artifact@v8"),
        "release workflow should use a Node 24-ready download-artifact release"
    );
}

#[test]
fn publish_job_checks_out_repo_before_verifying_tag() {
    let workflow = fs::read_to_string(".github/workflows/release.yml")
        .expect("release workflow should be readable");
    let (_, publish_section) = workflow
        .split_once("publish:")
        .expect("release workflow should define a publish job");

    assert!(
        publish_section.contains("actions/checkout@v5"),
        "publish job should check out the repo before running gh release create --verify-tag"
    );
}
