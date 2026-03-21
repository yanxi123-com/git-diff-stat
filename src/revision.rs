use crate::cli::Cli;
use crate::git::Git;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionSelection {
    WorkingTree,
    CommitPatch(String),
    Revisions(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionEndpoints {
    pub old: String,
    pub new: String,
}

impl RevisionSelection {
    pub fn from_cli(cli: &Cli) -> Result<Self, String> {
        if let Some(commit) = &cli.commit {
            return Self::from_commit(commit).map_err(str::to_string);
        }

        if cli.last {
            return Ok(Self::CommitPatch("HEAD".to_string()));
        }

        if cli.revisions.is_empty() {
            return Ok(Self::WorkingTree);
        }

        Ok(Self::Revisions(cli.revisions.clone()))
    }

    pub fn from_commit(revision: &str) -> Result<Self, &'static str> {
        if revision.trim().is_empty() {
            return Err("commit revision cannot be empty");
        }

        Ok(Self::CommitPatch(revision.to_string()))
    }

    pub fn git_diff_args(&self) -> Vec<String> {
        match self {
            Self::WorkingTree => Vec::new(),
            Self::CommitPatch(revision) => vec![format!("{revision}^!")],
            Self::Revisions(revisions) if revisions.len() == 1 && !revisions[0].contains("..") => {
                vec![revisions[0].clone(), "HEAD".to_string()]
            }
            Self::Revisions(revisions) => revisions.clone(),
        }
    }

    pub fn endpoints(&self, git: &Git) -> Result<Option<RevisionEndpoints>, String> {
        match self {
            Self::WorkingTree => Ok(None),
            Self::CommitPatch(revision) => Ok(Some(RevisionEndpoints {
                old: format!("{revision}^"),
                new: revision.clone(),
            })),
            Self::Revisions(revisions) => resolve_revision_endpoints(git, revisions).map(Some),
        }
    }

    pub fn describe_scope(&self, git: &Git, last_flag: bool) -> Result<String, String> {
        match self {
            Self::WorkingTree => Ok("in the working tree".to_string()),
            Self::CommitPatch(revision) => {
                if last_flag {
                    Ok("in the last commit".to_string())
                } else {
                    Ok(format!("in commit {revision}"))
                }
            }
            Self::Revisions(_) => {
                let endpoints = self
                    .endpoints(git)?
                    .ok_or_else(|| "missing revision endpoints".to_string())?;
                Ok(format!("from {} to {}", endpoints.old, endpoints.new))
            }
        }
    }
}

fn resolve_revision_endpoints(
    git: &Git,
    revisions: &[String],
) -> Result<RevisionEndpoints, String> {
    match revisions {
        [range] if range.contains("...") => {
            let mut parts = range.splitn(2, "...");
            let left = parts.next().unwrap_or_default();
            let right = parts.next().unwrap_or_default();
            if left.is_empty() || right.is_empty() {
                return Err(format!("invalid revision range: {range}"));
            }

            Ok(RevisionEndpoints {
                old: git.merge_base(left, right)?,
                new: right.to_string(),
            })
        }
        [range] if range.contains("..") => {
            let mut parts = range.splitn(2, "..");
            let left = parts.next().unwrap_or_default();
            let right = parts.next().unwrap_or_default();
            if left.is_empty() || right.is_empty() {
                return Err(format!("invalid revision range: {range}"));
            }

            Ok(RevisionEndpoints {
                old: left.to_string(),
                new: right.to_string(),
            })
        }
        [single] => Ok(RevisionEndpoints {
            old: single.clone(),
            new: "HEAD".to_string(),
        }),
        [old, new] => Ok(RevisionEndpoints {
            old: old.clone(),
            new: new.clone(),
        }),
        _ => Err("expected zero, one, or two revisions".to_string()),
    }
}
