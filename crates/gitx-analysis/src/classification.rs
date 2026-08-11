use gitx_core::types::CommitClassification;

/// Heuristics for classifying a commit based on its message.
/// Classification must be explicitly labeled as heuristic in the UI.
pub fn classify_commit_message(message: &str) -> CommitClassification {
    let lower = message.to_lowercase();

    // Check for explicit bug fixes and patches
    if lower.starts_with("fix:")
        || lower.contains("bug")
        || lower.contains("hotfix")
        || lower.contains("regression")
        || lower.contains("patch")
    {
        return CommitClassification::Fix;
    }

    if lower.starts_with("feat:") || lower.starts_with("feature:") {
        return CommitClassification::Feature;
    }

    if lower.starts_with("refactor:") {
        return CommitClassification::Refactor;
    }

    if lower.starts_with("docs:") {
        return CommitClassification::Docs;
    }

    if lower.starts_with("test:") || lower.starts_with("tests:") {
        return CommitClassification::Test;
    }

    if lower.starts_with("chore:") {
        return CommitClassification::Chore;
    }

    if lower.starts_with("revert") {
        return CommitClassification::Revert;
    }

    if lower.starts_with("merge") {
        return CommitClassification::Merge;
    }

    CommitClassification::Unknown
}
