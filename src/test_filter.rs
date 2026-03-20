use std::collections::{HashMap, HashSet};

use crate::change::FileChange;
use crate::cli::TestFilterMode;
use crate::git::Git;
use crate::lang::{detect_language, python, rust};
use crate::patch::parse_patch;
use crate::render::DisplayStat;
use crate::revision::{RevisionEndpoints, RevisionSelection};

pub fn build_test_filtered_stats(
    git: &Git,
    selection: &RevisionSelection,
    changes: &[FileChange],
    mode: TestFilterMode,
) -> Result<Vec<DisplayStat>, String> {
    let patch_output = git.diff_patch(&selection.git_diff_args())?;
    let patch = parse_patch(&patch_output)?;
    let patch_map = patch
        .files
        .into_iter()
        .map(|file| (file.path.clone(), file))
        .collect::<HashMap<_, _>>();
    let endpoints = selection.endpoints(git)?;
    let whole_test_paths = build_whole_test_paths(git, endpoints.as_ref())?;
    let context = BuildContext {
        git,
        endpoints: &endpoints,
        patch_map: &patch_map,
        mode,
    };
    let mut stats = Vec::new();

    for change in changes {
        if change.added + change.deleted == 0 {
            continue;
        }

        let Some(language) = change_language(change) else {
            continue;
        };

        let (added, deleted) = match language {
            "rs" => build_counts_for_rust(&context, &whole_test_paths, change)?,
            "py" => build_counts_for_python(&context, &whole_test_paths, change)?,
            _ => continue,
        };

        if added + deleted == 0 {
            continue;
        }

        stats.push(DisplayStat {
            path: change.path.clone(),
            added,
            deleted,
        });
    }

    Ok(stats)
}

struct WholeTestPaths {
    old: HashMap<&'static str, HashSet<String>>,
    new: HashMap<&'static str, HashSet<String>>,
}

struct BuildContext<'a> {
    git: &'a Git,
    endpoints: &'a Option<RevisionEndpoints>,
    patch_map: &'a HashMap<String, crate::patch::FilePatch>,
    mode: TestFilterMode,
}

fn build_whole_test_paths(
    git: &Git,
    endpoints: Option<&RevisionEndpoints>,
) -> Result<WholeTestPaths, String> {
    let (old_sources, new_sources) = match endpoints {
        Some(endpoints) => (
            load_revision_sources(git, &endpoints.old)?,
            load_revision_sources(git, &endpoints.new)?,
        ),
        None => (load_index_sources(git)?, load_worktree_sources(git)?),
    };

    let mut old = HashMap::new();
    old.insert("rs", rust::collect_whole_test_paths(&old_sources)?);
    old.insert("py", python::collect_whole_test_paths(&old_sources)?);

    let mut new = HashMap::new();
    new.insert("rs", rust::collect_whole_test_paths(&new_sources)?);
    new.insert("py", python::collect_whole_test_paths(&new_sources)?);

    Ok(WholeTestPaths { old, new })
}

fn build_counts_for_rust(
    context: &BuildContext<'_>,
    whole_test_paths: &WholeTestPaths,
    change: &FileChange,
) -> Result<(usize, usize), String> {
    build_counts(
        context,
        whole_test_paths.old.get("rs"),
        whole_test_paths.new.get("rs"),
        change,
        rust::split_untracked_source,
        rust::split_file_patch,
    )
}

fn build_counts_for_python(
    context: &BuildContext<'_>,
    whole_test_paths: &WholeTestPaths,
    change: &FileChange,
) -> Result<(usize, usize), String> {
    build_counts(
        context,
        whole_test_paths.old.get("py"),
        whole_test_paths.new.get("py"),
        change,
        python::split_untracked_source,
        python::split_file_patch,
    )
}

fn build_counts<Split, UntrackedFn, PatchFn>(
    context: &BuildContext<'_>,
    old_whole_test_paths: Option<&HashSet<String>>,
    new_whole_test_paths: Option<&HashSet<String>>,
    change: &FileChange,
    split_untracked: UntrackedFn,
    split_patch: PatchFn,
) -> Result<(usize, usize), String>
where
    Split: TestSplitCounts,
    UntrackedFn: Fn(&str) -> Result<Split, String>,
    PatchFn: Fn(&crate::patch::FilePatch, &str, &str) -> Result<Split, String>,
{
    let old_is_whole_test = old_whole_test_paths
        .map(|paths| paths.contains(&change.old_path))
        .unwrap_or(false);
    let new_is_whole_test = new_whole_test_paths
        .map(|paths| paths.contains(&change.new_path))
        .unwrap_or(false);

    if old_is_whole_test || new_is_whole_test {
        return Ok(select_counts_from_whole_file(
            change,
            old_is_whole_test,
            new_is_whole_test,
            context.mode,
        ));
    }

    if change.untracked {
        let source = context.git.read_worktree_file(&change.new_path)?;
        let split = split_untracked(&source)?;
        return Ok(select_counts_from_split(&split, context.mode));
    }

    let file_patch = context
        .patch_map
        .get(&change.new_path)
        .ok_or_else(|| format!("missing patch data for {}", change.path))?;
    let old_source = match context.endpoints {
        Some(endpoints) => context
            .git
            .show_file_at_revision(&endpoints.old, &change.old_path)
            .unwrap_or_default(),
        None => context
            .git
            .show_index_file(&change.old_path)
            .unwrap_or_default(),
    };
    let new_source = match context.endpoints {
        Some(endpoints) => context
            .git
            .show_file_at_revision(&endpoints.new, &change.new_path)
            .unwrap_or_default(),
        None => context
            .git
            .read_worktree_file(&change.new_path)
            .unwrap_or_default(),
    };
    let split = split_patch(file_patch, &old_source, &new_source)?;
    Ok(select_counts_from_split(&split, context.mode))
}

trait TestSplitCounts {
    fn test_added(&self) -> usize;
    fn test_deleted(&self) -> usize;
    fn non_test_added(&self) -> usize;
    fn non_test_deleted(&self) -> usize;
}

impl TestSplitCounts for crate::filter::RustTestSplit {
    fn test_added(&self) -> usize {
        self.test_added
    }

    fn test_deleted(&self) -> usize {
        self.test_deleted
    }

    fn non_test_added(&self) -> usize {
        self.non_test_added
    }

    fn non_test_deleted(&self) -> usize {
        self.non_test_deleted
    }
}

impl TestSplitCounts for crate::python_tests::PythonTestSplit {
    fn test_added(&self) -> usize {
        self.test_added
    }

    fn test_deleted(&self) -> usize {
        self.test_deleted
    }

    fn non_test_added(&self) -> usize {
        self.non_test_added
    }

    fn non_test_deleted(&self) -> usize {
        self.non_test_deleted
    }
}

fn select_counts_from_whole_file(
    change: &FileChange,
    old_is_whole_test: bool,
    new_is_whole_test: bool,
    mode: TestFilterMode,
) -> (usize, usize) {
    match mode {
        TestFilterMode::TestOnly => (
            if new_is_whole_test { change.added } else { 0 },
            if old_is_whole_test { change.deleted } else { 0 },
        ),
        TestFilterMode::NonTestOnly => (
            if new_is_whole_test { 0 } else { change.added },
            if old_is_whole_test { 0 } else { change.deleted },
        ),
        TestFilterMode::All => (change.added, change.deleted),
    }
}

fn select_counts_from_split(split: &impl TestSplitCounts, mode: TestFilterMode) -> (usize, usize) {
    match mode {
        TestFilterMode::TestOnly => (split.test_added(), split.test_deleted()),
        TestFilterMode::NonTestOnly => (split.non_test_added(), split.non_test_deleted()),
        TestFilterMode::All => (
            split.test_added() + split.non_test_added(),
            split.test_deleted() + split.non_test_deleted(),
        ),
    }
}

fn change_language(change: &FileChange) -> Option<&'static str> {
    detect_language(&change.new_path).or_else(|| detect_language(&change.old_path))
}

fn load_index_sources(git: &Git) -> Result<Vec<(String, String)>, String> {
    load_sources(git.tracked_files()?, |path| git.show_index_file(path))
}

fn load_worktree_sources(git: &Git) -> Result<Vec<(String, String)>, String> {
    let mut paths = git.tracked_files()?;
    paths.retain(|path| git.worktree_file_exists(path));
    paths.extend(git.untracked_files()?);
    load_sources(paths, |path| git.read_worktree_file(path))
}

fn load_revision_sources(git: &Git, revision: &str) -> Result<Vec<(String, String)>, String> {
    load_sources(git.revision_files(revision)?, |path| {
        git.show_file_at_revision(revision, path)
    })
}

fn load_sources<F>(paths: Vec<String>, mut read_source: F) -> Result<Vec<(String, String)>, String>
where
    F: FnMut(&str) -> Result<String, String>,
{
    let mut sources = Vec::new();

    for path in paths {
        if detect_language(&path).is_none() {
            continue;
        }

        sources.push((path.clone(), read_source(&path)?));
    }

    Ok(sources)
}
