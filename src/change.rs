use std::fs;

use crate::git::Git;
use crate::revision::RevisionSelection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub old_path: String,
    pub new_path: String,
    pub added: usize,
    pub deleted: usize,
    pub untracked: bool,
}

pub fn collect_working_tree_changes(git: &Git) -> Result<Vec<FileChange>, String> {
    let mut changes = parse_numstat_output(&git.diff_numstat(&[])?)?;

    for path in git.untracked_files()? {
        changes.push(FileChange {
            old_path: path.clone(),
            new_path: path.clone(),
            added: git.file_line_count(&path)?,
            deleted: 0,
            path,
            untracked: true,
        });
    }

    Ok(changes)
}

pub fn collect_changes(
    git: &Git,
    selection: &RevisionSelection,
) -> Result<Vec<FileChange>, String> {
    match selection {
        RevisionSelection::WorkingTree => collect_working_tree_changes(git),
        _ => parse_numstat_output(&git.diff_numstat(&selection.git_diff_args())?),
    }
}

fn parse_numstat_line(line: &str) -> Result<FileChange, String> {
    let mut parts = line.splitn(3, '\t');
    let added = parse_numstat_count(
        parts
            .next()
            .ok_or_else(|| format!("missing added count in numstat line: {line}"))?,
        "added",
    )?;
    let deleted = parse_numstat_count(
        parts
            .next()
            .ok_or_else(|| format!("missing deleted count in numstat line: {line}"))?,
        "deleted",
    )?;
    let path = parts
        .next()
        .ok_or_else(|| format!("missing path in numstat line: {line}"))?;
    let (old_path, new_path) = parse_numstat_paths(path);

    Ok(FileChange {
        path: path.to_string(),
        old_path,
        new_path,
        added,
        deleted,
        untracked: false,
    })
}

fn parse_numstat_count(value: &str, label: &str) -> Result<usize, String> {
    if value == "-" {
        return Ok(0);
    }

    value
        .parse::<usize>()
        .map_err(|error| format!("invalid {label} count in numstat line: {error}"))
}

fn parse_numstat_paths(path: &str) -> (String, String) {
    if let Some((prefix, rest)) = path.split_once('{')
        && let Some((middle, suffix)) = rest.split_once('}')
        && let Some((old_segment, new_segment)) = middle.split_once(" => ")
    {
        return (
            format!("{prefix}{old_segment}{suffix}"),
            format!("{prefix}{new_segment}{suffix}"),
        );
    }

    if let Some((old_path, new_path)) = path.split_once(" => ") {
        return (old_path.to_string(), new_path.to_string());
    }

    (path.to_string(), path.to_string())
}

fn parse_numstat_output(output: &str) -> Result<Vec<FileChange>, String> {
    output
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_numstat_line)
        .collect()
}

pub fn line_count(contents: &str) -> usize {
    if contents.is_empty() {
        return 0;
    }

    contents.lines().count()
}

pub fn file_line_count(path: &std::path::Path) -> Result<usize, String> {
    let contents = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    Ok(line_count(&contents))
}

#[cfg(test)]
mod tests {
    use super::parse_numstat_output;

    #[test]
    fn treats_binary_numstat_counts_as_zero() {
        let changes = parse_numstat_output("-\t-\tassets/logo.png\n").unwrap();

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "assets/logo.png");
        assert_eq!(changes[0].added, 0);
        assert_eq!(changes[0].deleted, 0);
    }
}
