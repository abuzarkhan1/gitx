# Analysis Engine

## 1. Principle

Analysis must be deterministic, measurable, and explainable.

## 2. Metrics

### Change frequency

Number of commits affecting an entity.

### Churn

Total additions + deletions over a defined period.

### Recent churn

Churn inside a configurable recent window.

### Contributor count

Number of distinct contributors affecting the entity.

### Bug-fix frequency

Number of heuristically classified fix/revert commits affecting the entity.

### File age

Time since first introduction.

### Activity recency

Time since last relevant change.

### Structural complexity

Before a language parser exists, use cheap proxy signals:

```text
LOC
file size
function count (heuristic)
change frequency
churn
```

Later, replace or augment with AST-derived metrics via language analyzers (Tree-sitter). Never let missing complexity inputs silently zero out a score; mark them unavailable.

## 3. Hotspot model

A hotspot is a file or area with unusually high change/maintenance activity.

Recommended signals:

```text
change frequency
recent churn
bug-fix frequency
contributor count
ownership concentration
optional structural complexity
```

Normalize signals to 0–100 before weighting.

Example:

```text
score =
    0.25 * change_frequency +
    0.20 * recent_churn +
    0.20 * bug_fix_frequency +
    0.15 * ownership_concentration +
    0.20 * complexity
```

Weights must be configuration-driven and documented.

Normalized scores map to classification bands:

```text
0–30    LOW
31–60   MEDIUM
61–80   HIGH
81–100  CRITICAL
```

Bands are presentational and must be derived from the same normalized 0–100 value used by the score.

Do not claim this score predicts defects. It is a change/maintenance-risk indicator, and should be labeled as such rather than as "bug probability".

## 4. Ownership model

Contribution weighting should consider:

- commit count
- changed lines
- recency

A basic configurable model:

```text
ownership =
    weighted_change_contribution
```

The UI should expose the chosen calculation.

Additional ownership signals:

- **subsystem ownership** — aggregate ownership per directory/module, not only per file
- **knowledge concentration** — files or areas dominated by a single contributor (bus-factor risk)
- **inactive ownership** — areas whose primary contributor has not been active within a configurable window

Each signal must be computed deterministically and expose its evidence like any other metric.

## 5. Branch intelligence

Calculate:

```text
ahead
behind
common ancestor
diverged commits
branch age
recent activity
shared files
```

Merge complexity is an estimate and must be labeled as such.

A useful first model:

```text
merge_complexity =
    weighted overlap of changed files
    +
    number of diverged commits
    +
    overlapping directories
```

Do not present it as a guarantee of merge conflicts.

## 6. Risk

Risk should be a composite maintenance signal.

Example:

```text
risk =
    hotspot_score
    +
    ownership concentration
    +
    recent churn
    +
    structural complexity
```

Always show evidence.

## 7. Commit classification

Heuristics may inspect:

- message prefixes
- conventional-commit type
- keywords
- changed paths

Example:

```text
fix:
bug
hotfix
regression
patch
```

Classification must be marked heuristic.

## 8. Repository health

Repository health is a composite, deterministic overview derived from measurable signals. It is a maintenance signal, never an assessment of team or product quality.

```text
overall = weighted aggregation of six sub-scores
```

Each sub-score is normalized to 0–100, uses the **health** classification bands (health is higher-is-better, so the labels run the opposite direction of the risk bands), and must carry its own evidence:

```text
0–30    POOR
31–60   FAIR
61–80   GOOD
81–100  EXCELLENT
```

- **code hotspots** — distribution and severity of high-change/high-risk files
- **ownership risk** — ownership concentration, knowledge concentration, inactive ownership
- **branch hygiene** — stale branches, divergence, ahead/behind, shared-file overlap
- **change volatility** — churn, change frequency, activity recency
- **architecture stability** — structural churn, added/removed/moved modules, dependency-change frequency
- **recovery risk** — volume and age of recoverable work, reflog churn

Weights are configuration-driven and documented. Every sub-score must satisfy the explainability contract; a health number without evidence is a bug.

## 9. Bug and regression history

Build on fix/revert classification to surface recurring problem areas:

- files repeatedly involved in fix-classified commits
- reverts that follow shortly after the change they revert (potential regression)
- fix commits touching files that were previously fixed multiple times
- areas whose fix density is high relative to overall activity

These are evidence lists to guide investigation, not predictions. All classification heuristics must remain explicitly labeled.

## 10. Architecture analysis

Start with:

- directory hierarchy
- file relationships
- dependency manifests
- import/module relationships where available

Later add language-aware parsing.

Architecture comparison should detect:

- added areas
- removed areas
- moved files
- new dependencies
- removed dependencies
- changed dependency relationships
- architectural milestones (initial commit, first release tag, major module additions, large structural refactors, dependency-direction changes)

## 11. Dependency analysis

Support common manifest formats incrementally.

Examples:

```text
Cargo.toml
package.json
pyproject.toml
requirements.txt
go.mod
```

Dependency extraction is a plugin/adapter concern.

Track:

- dependency added
- dependency removed
- version changed
- direct/indirect where reliably available
- version evolution — timeline of version changes per dependency
- dependency usage — which files/areas reference a dependency
- dependency churn — commits that change a dependency's declaration or usage

## 12. Release analysis

Compare two refs:

```text
v1.0.0 → v2.0.0
```

Show:

- commits
- files
- contributors
- additions
- deletions
- classifications
- top affected areas
- top hotspots introduced/changed

## 13. Explainability contract

Every score should have:

```text
value
classification
formula/version
inputs
source entities
time window
```

If an input is unavailable, do not silently substitute a fake value.
