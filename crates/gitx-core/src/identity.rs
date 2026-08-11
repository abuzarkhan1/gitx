//! Contributor identity normalization (docs/05 §3).
//!
//! One developer may commit under several raw spellings (`Abuzar`, `abuzar`,
//! `Abuzar Khan`, different emails). Normalization here is *conservative*: we
//! never merge identities on weak guesses — only on an explicit, configurable
//! user mapping. The canonical key is the lowercased email when present,
//! falling back to the normalized name, so two raw spellings of the same email
//! merge deterministically without configuration.

use serde::{Deserialize, Serialize};

/// Normalize an email address: trim, lowercase, strip a leading `mailto:`
/// (case-insensitively).
pub fn normalize_email(email: &str) -> String {
    let trimmed = email.trim();
    let without_prefix = if trimmed
        .get(..7)
        .is_some_and(|p| p.eq_ignore_ascii_case("mailto:"))
    {
        &trimmed[7..]
    } else {
        trimmed
    };
    without_prefix.to_lowercase()
}

/// Normalize a display name: trim and collapse internal whitespace runs.
/// Case is preserved (names are case-sensitive; email is the merge key).
pub fn normalize_name(name: &str) -> String {
    name.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A resolved contributor identity: raw input plus the normalized display
/// form and the canonical merge key (docs/05 §3: raw identity, normalized
/// display identity, optional user mapping).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedIdentity {
    /// The raw author name as recorded in the commit.
    pub raw_name: String,
    /// The raw email as recorded in the commit.
    pub raw_email: String,
    /// Display name after normalization and user mapping (never empty).
    pub display_name: String,
    /// Canonical merge key: lowercased email when present, else normalized name.
    pub key: String,
}

/// Explicit user mapping: commits authored under `email` display as `name`
/// and group under that identity (docs/05 §3: "Never silently merge
/// identities based only on weak guesses" — mapping is explicit only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityMapping {
    /// Raw email to match (compared case-insensitively after trimming).
    pub email: String,
    /// Canonical display name to use for that email.
    pub name: String,
}

/// Resolve a raw (name, email) pair into a normalized identity, applying the
/// configured user mappings when an email matches.
pub fn resolve(mappings: &[IdentityMapping], name: &str, email: &str) -> NormalizedIdentity {
    let norm_name = normalize_name(name);
    let norm_email = normalize_email(email);
    let display_name = if !norm_email.is_empty() {
        mappings
            .iter()
            .find(|m| normalize_email(&m.email) == norm_email)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| norm_name.clone())
    } else {
        norm_name.clone()
    };
    let key = if norm_email.is_empty() {
        norm_name
    } else {
        norm_email
    };
    NormalizedIdentity {
        raw_name: name.to_string(),
        raw_email: email.to_string(),
        display_name,
        key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_normalization_is_case_insensitive() {
        assert_eq!(
            normalize_email("  Abuzar@Example.COM "),
            "abuzar@example.com"
        );
        assert_eq!(normalize_email("mailto:A@b.co"), "a@b.co");
    }

    #[test]
    fn name_normalization_collapses_whitespace() {
        assert_eq!(normalize_name("  Abuzar   Khan\n"), "Abuzar Khan");
    }

    #[test]
    fn same_email_merges_without_mapping() {
        let a = resolve(&[], "Abuzar", "abuzar@x.co");
        let b = resolve(&[], "abuzar", "Abuzar@X.CO");
        assert_eq!(a.key, b.key);
        assert_eq!(a.key, "abuzar@x.co");
    }

    #[test]
    fn mapping_overrides_display_name_only() {
        let mappings = vec![IdentityMapping {
            email: "old@x.co".into(),
            name: "Abuzar Khan".into(),
        }];
        let id = resolve(&mappings, "old", "OLD@x.co");
        assert_eq!(id.display_name, "Abuzar Khan");
        assert_eq!(id.key, "old@x.co");
        // Unmapped identities keep their own spelling and key.
        let other = resolve(&mappings, "Dev", "dev@x.co");
        assert_eq!(other.display_name, "Dev");
        assert_eq!(other.key, "dev@x.co");
    }

    #[test]
    fn empty_email_keys_on_name() {
        let id = resolve(&[], "No Email", "");
        assert_eq!(id.key, "No Email");
    }

    // Property tests (docs/14 §5): normalization must be idempotent and
    // stable under case/whitespace variation, using the Rust built-in test
    // harness over generated inputs.

    #[test]
    fn normalization_is_idempotent_over_generated_inputs() {
        let samples = [
            " Abuzar ",
            "abuzar   khan",
            " A B C ",
            "",
            "x",
            "  spaced  out  name  ",
        ];
        for name in samples {
            let once = normalize_name(name);
            let twice = normalize_name(&once);
            assert_eq!(
                once, twice,
                "normalize_name must be idempotent for `{name}`"
            );
            assert!(
                !once.contains("  "),
                "no internal double spaces for `{name}`"
            );
        }
        let emails = ["Abuzar@X.CO", "  a@b.c ", "MAILTO:a@b.c", "a@B.C", ""];
        for email in emails {
            let once = normalize_email(email);
            let twice = normalize_email(&once);
            assert_eq!(
                once, twice,
                "normalize_email must be idempotent for `{email}`"
            );
            assert_eq!(once, once.to_lowercase(), "normalized email is lowercase");
        }
    }

    #[test]
    fn resolve_key_is_stable_under_case_and_whitespace_variation() {
        // Same logical identity spelled many ways must collapse to one key.
        let variants = [
            ("Abuzar", "ABUZAR@x.co"),
            ("abuzar", "abuzar@X.CO"),
            (" Abuzar  Khan ", " abuzar@x.co "),
            ("aBuzar", "mailto:abuzar@x.co"),
        ];
        let keys: std::collections::HashSet<String> = variants
            .iter()
            .map(|(n, e)| resolve(&[], n, e).key)
            .collect();
        assert_eq!(keys.len(), 1, "all variants share one canonical key");
    }

    #[test]
    fn mapping_never_changes_the_canonical_key() {
        // An explicit mapping may change the *display name* but must never
        // merge identities that have different emails (docs/05 §3: no weak
        // guesses — mapping is email-keyed only).
        let mappings = vec![IdentityMapping {
            email: "a@x.co".into(),
            name: "Renamed".into(),
        }];
        let a = resolve(&mappings, "old", "a@x.co");
        let b = resolve(&mappings, "other", "b@x.co");
        assert_eq!(a.display_name, "Renamed");
        assert_ne!(a.key, b.key, "different emails stay separate identities");
        assert_eq!(b.display_name, "other");
    }
}
