# Data Flow and Algorithms

## 1. Repository intelligence pipeline

```text
.git
 ↓
Git object/ref reader
 ↓
normalized Git domain
 ↓
persistent index
 ↓
derived metrics
 ↓
analysis results
 ↓
CLI/TUI
```

## 2. Commit graph

Represent:

```text
Commit → Parents
```

This supports:

- ancestry
- divergence
- merge-base calculations
- timeline graphs
- branch analysis

## 3. File lineage

Track:

```text
file path
→ change events
→ rename events
→ previous path
→ next path
```

Rename detection must distinguish:

- Git-provided rename information
- GitX inference

Deleted files remain part of file archaeology: the last known content of a deleted file is recoverable from Git objects.

## 4. Churn

Basic:

```text
churn = insertions + deletions
```

Recent churn:

```text
sum(churn for changes within configured time window)
```

## 5. Ownership

Possible base metric:

```text
contribution_weight =
    changed_lines × recency_weight
```

Aggregate by contributor.

Weights must be documented and configurable.

## 6. Hotspot

Hotspot is a multi-signal ranking.

Do not equate hotspot with bug probability.

## 7. Branch divergence

Given branch B and base M:

```text
ahead  = commits reachable from B but not M
behind = commits reachable from M but not B
```

Common ancestor should be computed from actual Git ancestry.

## 8. Merge overlap

For an estimate:

```text
changed_files(branch_A)
INTERSECT
changed_files(branch_B)
```

Then weight overlap by:

- number of files
- line churn where available
- directory concentration

Result:

```text
estimated merge complexity
```

## 9. Architecture comparison

Represent snapshots as:

```text
nodes = files/directories/modules
edges = dependency/import relationships
```

Compare:

```text
added nodes
removed nodes
changed edges
moved nodes
```

## 10. Risk explanation

A risk panel should say:

```text
High maintenance risk

Evidence:
- 47 historical changes
- 18 changes in last 30 days
- 9 fix-classified commits
- 91% ownership concentration
- high structural complexity
```

This is preferable to a mysterious number.

## 11. Determinism

All algorithms must avoid:

- random ranking
- current time without explicit time-window semantics
- machine-specific ordering
- unstable hash-map iteration in user-visible output

Sort ties deterministically.

## 12. Repository health

Aggregate the six health sub-scores from their underlying metrics:

```text
overall = Σ(weight_i × sub_score_i)
```

Each sub-score must itself be explainable and use the documented classification bands. Health is a maintenance signal, never a judgement of the team or product.

## 13. Line-level history

Line-level (blame-style) attribution should be built from per-commit diffs or a dedicated blame engine, not by shelling out to `git blame`.

Compute lazily and paginate; never compute line attribution for every file during indexing.
