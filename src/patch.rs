#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    pub files: Vec<FilePatch>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilePatch {
    pub path: String,
    pub line_events: Vec<LineEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Added,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineEvent {
    pub kind: LineKind,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
}

pub fn parse_patch(input: &str) -> Result<Patch, String> {
    let mut files = Vec::new();
    let mut current_path: Option<String> = None;
    let mut current_events = Vec::new();
    let mut pending_old_path: Option<String> = None;
    let mut old_line = 0usize;
    let mut new_line = 0usize;
    let mut in_hunk = false;

    for line in input.lines() {
        if let Some(header) = line.strip_prefix("diff --git a/") {
            pending_old_path = Some(parse_diff_git_old_path(header)?.to_string());
            continue;
        }

        if let Some(path) = line.strip_prefix("+++ b/") {
            if let Some(previous_path) = current_path.replace(path.to_string()) {
                files.push(FilePatch {
                    path: previous_path,
                    line_events: std::mem::take(&mut current_events),
                });
            }
            in_hunk = false;
            continue;
        }

        if line == "+++ /dev/null" {
            let deleted_path = pending_old_path
                .clone()
                .ok_or_else(|| "missing deleted file path in diff header".to_string())?;
            if let Some(previous_path) = current_path.replace(deleted_path) {
                files.push(FilePatch {
                    path: previous_path,
                    line_events: std::mem::take(&mut current_events),
                });
            }
            in_hunk = false;
            continue;
        }

        if let Some(header) = line.strip_prefix("@@ ") {
            let (old_start, new_start) = parse_hunk_header(header)?;
            old_line = old_start;
            new_line = new_start;
            in_hunk = true;
            continue;
        }

        if !in_hunk {
            continue;
        }

        if line.starts_with('+') && !line.starts_with("+++") {
            current_events.push(LineEvent {
                kind: LineKind::Added,
                old_line: None,
                new_line: Some(new_line),
            });
            new_line += 1;
            continue;
        }

        if line.starts_with('-') && !line.starts_with("---") {
            current_events.push(LineEvent {
                kind: LineKind::Deleted,
                old_line: Some(old_line),
                new_line: None,
            });
            old_line += 1;
            continue;
        }

        if line.starts_with(' ') {
            old_line += 1;
            new_line += 1;
            continue;
        }
    }

    if let Some(path) = current_path {
        files.push(FilePatch {
            path,
            line_events: current_events,
        });
    }

    Ok(Patch { files })
}

fn parse_diff_git_old_path(header: &str) -> Result<&str, String> {
    header
        .split_once(" b/")
        .map(|(old_path, _)| old_path)
        .ok_or_else(|| format!("invalid diff header: diff --git a/{header}"))
}

fn parse_hunk_header(header: &str) -> Result<(usize, usize), String> {
    let end = header
        .find(" @@")
        .ok_or_else(|| format!("invalid hunk header: @@ {header}"))?;
    let ranges = &header[..end];
    let mut parts = ranges.split(' ');
    let old_range = parts
        .next()
        .ok_or_else(|| format!("missing old range in hunk header: {header}"))?;
    let new_range = parts
        .next()
        .ok_or_else(|| format!("missing new range in hunk header: {header}"))?;

    Ok((parse_range_start(old_range)?, parse_range_start(new_range)?))
}

fn parse_range_start(range: &str) -> Result<usize, String> {
    let start = range
        .trim_start_matches(['-', '+'])
        .split(',')
        .next()
        .ok_or_else(|| format!("invalid range: {range}"))?;
    start
        .parse::<usize>()
        .map_err(|error| format!("invalid range start in {range}: {error}"))
}
