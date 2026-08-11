# Testing Strategy

## 1. Test layers

```text
Unit
Integration
Fixture repositories
Snapshot
Property
Benchmark
End-to-end CLI
```

## 2. Unit tests

Test:

- metrics
- normalization
- scoring
- date windows
- branch calculations
- ownership calculations
- commit classification
- path handling
- serialization

## 3. Integration tests

Test against real Git repositories.

Cases:

- linear history
- merge commits
- branches
- tags
- renames
- deletions
- binary files
- empty commits
- revert commits
- rewritten branches

## 4. Fixture repositories

Create deterministic fixture repos under:

```text
tests/fixtures/
```

Each fixture should have documented expected behavior.

Suggested fixtures:

```text
linear/
merge-heavy/
rename-history/
branch-divergence/
reflog-recovery/
large-file-tree/
multi-author/
binary-files/
monorepo/
```

## 5. Snapshot tests

Use snapshots for:

- CLI human-readable output
- selected TUI views
- JSON schemas where useful

Snapshots must not contain timestamps or machine-specific paths unless normalized.

## 6. Property tests

Potential properties:

- commit parent graph is acyclic except for representation rules
- scores remain within configured ranges
- indexing twice without repository changes produces equivalent data
- serialization/deserialization preserves domain values
- search results are deterministic

## 7. End-to-end tests

Example:

```text
create fixture repo
→ run gitx scan
→ run gitx hotspots
→ run gitx search
→ run gitx recovery
→ inspect JSON
```

## 8. Index consistency test

Critical:

```text
fresh full index
```

must produce equivalent logical results to:

```text
incremental index from empty → history
```

## 9. Failure tests

Test:

- corrupted index
- missing `.git`
- invalid path
- interrupted indexing
- permission errors
- unsupported repository state
- malformed configuration

## 10. Performance tests

Benchmarks must cover realistic repositories, not only toy fixtures.

## 11. CI

CI should run:

- format check
- clippy
- unit tests
- integration tests
- snapshot checks
- build matrix
- selected benchmarks/regression checks
