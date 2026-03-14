#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RevisionSelection {
    CommitPatch(String),
    Revisions(Vec<String>),
}

impl RevisionSelection {
    pub fn from_commit(revision: &str) -> Result<Self, &'static str> {
        if revision.trim().is_empty() {
            return Err("commit revision cannot be empty");
        }

        Ok(Self::CommitPatch(revision.to_string()))
    }

    pub fn git_diff_args(&self) -> Vec<String> {
        match self {
            Self::CommitPatch(revision) => vec![format!("{revision}^!")],
            Self::Revisions(revisions) => revisions.clone(),
        }
    }
}
