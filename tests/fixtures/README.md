# Test fixtures

This directory documents the Git repositories used by the integration suite
(docs/14 §5). Fixtures are built **deterministically with the `git` CLI** —
the code under test never shells out to git; git is used only to *create*
repositories.

## Layout

- `build.sh` — builds the edge-case fixture repository (`edge-cases/`) used by
  `tests/integration/edge_cases.rs`: merge commits, binary files, empty
  commits, revert commits, and rewritten (amended) history.
- `README.md` — this file.
- Snapshot fixtures live in `../snapshots/` (blessed CLI output, docs/14 §5).

## Why not committed `.git` directories?

Git refuses to track nested repositories, so a committed fixture cannot
contain its own `.git/`. Instead:

1. The integration tests build each fixture at runtime in the system temp
   directory (`tests/common/mod.rs`, `FixtureRepo`), which keeps tests
   hermetic and parallel-safe.
2. `build.sh` reproduces the same fixture on demand for manual inspection or
   debugging — run it from this directory:

   ```sh
   ./build.sh            # builds edge-cases/ in place
   ./build.sh /tmp/fix   # or into a target directory
   ```

## Edge-case fixture contents (`build.sh`)

```
main
├── c1  add: initial files (text + binary)
├── c2  fix: modify text file
├── c3  (empty commit: no tree change)
├── c4  revert of c2
├── c5  merge feature/experiment (merge commit, 2 parents)
└── c6  (rewritten HEAD: `git commit --amend` after `git reset --soft`)
```

Used to verify: merge commits in the timeline, binary files in diffs,
empty commits with no changed files, revert detection (`gitx regressions`),
and rewritten-history handling (`gitx refresh`).
