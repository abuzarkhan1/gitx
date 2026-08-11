#[derive(Debug, Clone)]
pub struct Repository {
    pub id: i64,
    pub root_path: String,
    pub git_dir: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub normalized_name: Option<String>,
    pub normalized_email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Commit {
    pub oid: String,
    pub author_id: Option<i64>,
    pub committer_id: Option<i64>,
    pub tree_oid: Option<String>,
    pub timestamp: i64,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct CommitParent {
    pub commit_oid: String,
    pub parent_oid: String,
    pub parent_index: i32,
}

#[derive(Debug, Clone)]
pub struct File {
    pub id: i64,
    pub path: String,
    pub first_commit_oid: Option<String>,
    pub last_commit_oid: Option<String>,
    pub language: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub id: i64,
    pub commit_oid: String,
    pub file_id: i64,
    pub change_type: String,
    pub old_path: Option<String>,
    pub new_path: Option<String>,
    pub insertions: i32,
    pub deletions: i32,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub id: i64,
    pub name: String,
    pub tip_oid: Option<String>,
    pub is_remote: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub target_oid: String,
}

#[derive(Debug, Clone)]
pub struct ReflogEntry {
    pub id: i64,
    pub reference: String,
    pub old_oid: String,
    pub new_oid: String,
    pub actor: Option<String>,
    pub timestamp: Option<i64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileMetric {
    pub file_id: i64,
    pub commit_count: i64,
    pub total_insertions: i64,
    pub total_deletions: i64,
    pub recent_churn: i64,
    pub contributor_count: i64,
    pub bug_fix_count: i64,
    pub complexity_score: Option<f64>,
    pub hotspot_score: Option<f64>,
    pub updated_at: i64,
}
