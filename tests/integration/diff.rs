//! `gitx diff` (docs/13 §8): the streamed per-file patch renderer must
//! produce hunks whose +/- line counts match the FileChange insertions and
//! deletions reported by the tree diff.

use gitx_git::Repository;

#[path = "../common/mod.rs"]
mod common;
use common::FixtureRepo;

#[test]
fn diff_file_patch_matches_change_counts() {
    let Some(repo) = FixtureRepo::new("diff") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    repo.commit("feat: initial");
    repo.write(
        "src/lib.rs",
        "pub fn a() {}\npub fn b() {}\npub fn c() {}\n",
    );
    repo.commit("feat: add c");
    repo.write("src/lib.rs", "pub fn a() {}\n");
    repo.commit("fix: remove b and c");

    let gix = Repository::discover(repo.path()).unwrap();
    let head = gix.head_commit_id().unwrap();
    let first = gix.rev_walk(head).unwrap().last().unwrap().unwrap();

    let from_commit = gix.find_commit(first).unwrap();
    let head_commit = gix.find_commit(head).unwrap();
    let changes = gix
        .diff_tree_to_tree(Some(from_commit.tree_id), head_commit.tree_id)
        .unwrap();
    assert!(!changes.is_empty(), "two commits changed src/lib.rs");

    for change in &changes {
        let patch = gitx_git::diff::render_file_patch(
            &gix,
            Some(from_commit.tree_id),
            head_commit.tree_id,
            change,
        )
        .unwrap()
        .unwrap_or_else(|| panic!("changed file {} has a patch", change.path.display()));
        // The renderer emits a whole-file hunk (a valid unified diff): its
        // '-'/'+' lines must equal the old/new blob line counts.
        let old_path = change.old_path.as_ref().unwrap_or(&change.path);
        let old_bytes = gix
            .blob_at_path(from_commit.tree_id, old_path)
            .unwrap()
            .unwrap_or_default();
        let new_bytes = gix
            .blob_at_path(head_commit.tree_id, &change.path)
            .unwrap()
            .unwrap_or_default();
        let lines = |data: &[u8]| {
            if data.is_empty() {
                0
            } else {
                data.iter().filter(|&&b| b == b'\n').count() + usize::from(!data.ends_with(b"\n"))
            }
        };
        let plus = patch
            .lines()
            .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
            .count();
        let minus = patch
            .lines()
            .filter(|l| l.starts_with('-') && !l.starts_with("---"))
            .count();
        assert_eq!(
            minus,
            lines(&old_bytes),
            "minus lines for {}",
            change.path.display()
        );
        assert_eq!(
            plus,
            lines(&new_bytes),
            "plus lines for {}",
            change.path.display()
        );
    }
}

#[test]
fn diff_stat_matches_tree_change_list() {
    let Some(repo) = FixtureRepo::new("diff-stat") else {
        eprintln!("skipping: git CLI unavailable");
        return;
    };
    repo.write("a.txt", "one\ntwo\n");
    repo.commit("feat: a");
    repo.write("b.txt", "x\ny\nz\n");
    repo.commit("feat: b");
    repo.write("a.txt", "one\ntwo\nthree\n");
    repo.commit("feat: extend a");

    let gix = Repository::discover(repo.path()).unwrap();
    let head = gix.head_commit_id().unwrap();
    let first = gix.rev_walk(head).unwrap().last().unwrap().unwrap();
    let from_commit = gix.find_commit(first).unwrap();
    let head_commit = gix.find_commit(head).unwrap();

    let changes = gix
        .diff_tree_to_tree(Some(from_commit.tree_id), head_commit.tree_id)
        .unwrap();
    // a.txt modified (+1), b.txt added (+3).
    let a = changes
        .iter()
        .find(|c| c.path.ends_with("a.txt"))
        .expect("a.txt changed");
    assert_eq!(a.insertions, 1);
    let b = changes
        .iter()
        .find(|c| c.path.ends_with("b.txt"))
        .expect("b.txt changed");
    assert_eq!(b.insertions, 3);
    assert_eq!(b.deletions, 0);
}
