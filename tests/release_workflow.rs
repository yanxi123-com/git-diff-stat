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
