use gitx_git::Repository;
use gitx_git::models::{Commit, ObjectId};
use std::path::PathBuf;

#[derive(Default)]
pub struct TimelineOptions {
    pub max_count: Option<usize>,
    pub from: Option<ObjectId>,
    pub path: Option<PathBuf>,
    pub author: Option<String>,
    /// Only include commits at or after this unix timestamp.
    pub since: Option<i64>,
    /// Only include commits at or before this unix timestamp.
    pub until: Option<i64>,
    /// Only include commits whose committer name/email contains this
    /// (docs/02 V1 advanced filters).
    pub committer: Option<String>,
    /// Only include merge commits (2+ parents).
    pub merges_only: bool,
    /// Exclude merge commits.
    pub no_merges: bool,
}

pub struct HistoryService<'a> {
    pub repo: &'a Repository,
}

impl<'a> HistoryService<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    pub fn timeline(&self, options: TimelineOptions) -> anyhow::Result<Vec<Commit>> {
        let mut commits = Vec::new();

        let head_id = match options.from {
            Some(id) => id,
            None => self.repo.head_commit_id()?,
        };

        let mut count = 0;
        let limit = options.max_count.unwrap_or(usize::MAX);

        for commit_id_res in self.repo.rev_walk(head_id)? {
            if count >= limit {
                break;
            }
            let commit_id = commit_id_res?;
            let commit = self.repo.find_commit(commit_id)?;

            if let Some(author) = &options.author {
                let name_matches = commit.author.name.contains(author);
                let email_matches = commit.author.email.contains(author);
                if !name_matches && !email_matches {
                    continue;
                }
            }
            if let Some(committer) = &options.committer {
                let name_matches = commit.committer.name.contains(committer);
                let email_matches = commit.committer.email.contains(committer);
                if !name_matches && !email_matches {
                    continue;
                }
            }
            if options.merges_only && commit.parents.len() < 2 {
                continue;
            }
            if options.no_merges && commit.parents.len() >= 2 {
                continue;
            }
            if let Some(since) = options.since
                && commit.author.time < since
            {
                continue;
            }
            if let Some(until) = options.until
                && commit.author.time > until
            {
                continue;
            }
            if let Some(path) = &options.path
                && !self.commit_touches_path(&commit, path)?
            {
                continue;
            }

            commits.push(commit);
            count += 1;
        }

        Ok(commits)
    }

    /// Whether `commit` changed the given path, compared against its first
    /// parent (mainline). For a root commit, the file "changed" if it exists
    /// in the commit's tree.
    pub(crate) fn commit_touches_path(
        &self,
        commit: &Commit,
        path: &PathBuf,
    ) -> anyhow::Result<bool> {
        if commit.parents.is_empty() {
            return Ok(self.repo.blob_at_path(commit.tree_id, path)?.is_some());
        }
        let parent = self.repo.find_commit(commit.parents[0])?;
        let changes = self
            .repo
            .diff_tree_to_tree(Some(parent.tree_id), commit.tree_id)?;
        Ok(changes
            .iter()
            .any(|c| &c.path == path || c.old_path.as_ref() == Some(path)))
    }
}
