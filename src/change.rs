use std::fs;

use crate::git::Git;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub added: usize,
    pub deleted: usize,
    pub untracked: bool,
}

pub fn collect_working_tree_changes(git: &Git) -> Result<Vec<FileChange>, String> {
    let mut changes = git
        .diff_numstat(&[])?
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(parse_numstat_line)
        .collect::<Result<Vec<_>, _>>()?;

    for path in git.untracked_files()? {
        changes.push(FileChange {
            added: git.file_line_count(&path)?,
            deleted: 0,
            path,
            untracked: true,
        });
    }

    Ok(changes)
}

fn parse_numstat_line(line: &str) -> Result<FileChange, String> {
    let mut parts = line.splitn(3, '\t');
    let added = parts
        .next()
        .ok_or_else(|| format!("missing added count in numstat line: {line}"))?
        .parse::<usize>()
        .map_err(|error| format!("invalid added count in numstat line: {error}"))?;
    let deleted = parts
        .next()
        .ok_or_else(|| format!("missing deleted count in numstat line: {line}"))?
        .parse::<usize>()
        .map_err(|error| format!("invalid deleted count in numstat line: {error}"))?;
    let path = parts
        .next()
        .ok_or_else(|| format!("missing path in numstat line: {line}"))?;

    Ok(FileChange {
        path: path.to_string(),
        added,
        deleted,
        untracked: false,
    })
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
