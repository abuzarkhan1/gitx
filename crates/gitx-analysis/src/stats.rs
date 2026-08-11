use gitx_git::Repository;
use std::collections::HashMap;

/// High-level repository statistics (docs/08 overview panel, docs/07 `gitx stats`).
#[derive(Debug, Clone)]
pub struct RepoStats {
    pub commits: u64,
    pub contributors: usize,
    pub files: usize,
    pub branches: usize,
    pub tags: usize,
    /// Repository age in whole days (0 for a brand-new repo).
    pub age_days: i64,
    pub first_commit: Option<i64>,
    pub last_commit: Option<i64>,
    /// (extension, file count), sorted descending by count.
    pub languages: Vec<(String, usize)>,
    pub head_oid: Option<String>,
    pub head_message: Option<String>,
}

/// Compute repository statistics deterministically from Git data.
pub fn repository_stats(repo: &Repository) -> anyhow::Result<RepoStats> {
    let Some(head) = repo.head_commit_id().ok() else {
        return Ok(RepoStats {
            commits: 0,
            contributors: 0,
            files: 0,
            branches: repo.branches().unwrap_or_default().len(),
            tags: repo.tags().unwrap_or_default().len(),
            age_days: 0,
            first_commit: None,
            last_commit: None,
            languages: Vec::new(),
            head_oid: None,
            head_message: None,
        });
    };

    let head_commit = repo.find_commit(head)?;

    let mut commit_count = 0u64;
    let mut authors: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut first_commit_ts: Option<i64> = None;
    let mut last_commit_ts: Option<i64> = None;
    for id_res in repo.rev_walk(head)? {
        let commit = repo.find_commit(id_res?)?;
        commit_count += 1;
        authors.insert(format!("{} <{}>", commit.author.name, commit.author.email));
        first_commit_ts = Some(commit.author.time.min(first_commit_ts.unwrap_or(i64::MAX)));
        last_commit_ts = Some(commit.author.time.max(last_commit_ts.unwrap_or(i64::MIN)));
    }

    let files = repo.list_blobs(head_commit.tree_id)?;
    let mut languages: HashMap<String, usize> = HashMap::new();
    for path in &files {
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_else(|| "none".to_string());
        *languages.entry(ext).or_insert(0) += 1;
    }
    let mut language_list: Vec<(String, usize)> = languages.into_iter().collect();
    language_list.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let age_days = first_commit_ts
        .map(|t| (chrono::Utc::now().timestamp() - t) / 86_400)
        .unwrap_or(0);

    Ok(RepoStats {
        commits: commit_count,
        contributors: authors.len(),
        files: files.len(),
        branches: repo.branches().unwrap_or_default().len(),
        tags: repo.tags().unwrap_or_default().len(),
        age_days,
        first_commit: first_commit_ts,
        last_commit: last_commit_ts,
        languages: language_list,
        head_oid: Some(head.to_string()),
        head_message: Some(head_commit.message),
    })
}
