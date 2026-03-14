use std::path::{Path, PathBuf};
use std::process::Command;

use crate::change;

#[derive(Debug, Clone)]
pub struct Git {
    cwd: PathBuf,
}

impl Git {
    pub fn new(cwd: impl AsRef<Path>) -> Self {
        Self {
            cwd: cwd.as_ref().to_path_buf(),
        }
    }

    pub fn diff_numstat(&self, revision_args: &[String]) -> Result<String, String> {
        self.run_git(["diff", "--numstat"], revision_args)
    }

    pub fn untracked_files(&self) -> Result<Vec<String>, String> {
        let output = self.run_git(["ls-files", "--others", "--exclude-standard"], &[])?;
        Ok(output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(ToOwned::to_owned)
            .collect())
    }

    pub fn file_line_count(&self, path: &str) -> Result<usize, String> {
        change::file_line_count(&self.cwd.join(path))
    }

    fn run_git<const N: usize>(
        &self,
        base_args: [&str; N],
        revision_args: &[String],
    ) -> Result<String, String> {
        let output = Command::new("git")
            .args(base_args)
            .args(revision_args)
            .current_dir(&self.cwd)
            .output()
            .map_err(|error| format!("failed to execute git: {error}"))?;

        if output.status.success() {
            return String::from_utf8(output.stdout)
                .map_err(|error| format!("git returned non-utf8 stdout: {error}"));
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("not a git repository") {
            return Err("not a git repository".to_string());
        }

        Err(format!("git command failed: {}", stderr.trim()))
    }
}
