# Domain Model

## 1. Core entities

### Repository

```text
RepositoryId
root_path
git_dir
default_branch
head_commit
indexed_at
index_version
```

### Commit

```text
CommitId
author_id
committer_id
timestamp
message
tree_id
parents[]
```

### Author

```text
AuthorId
name
email_hash_or_normalized_identity
first_seen
last_seen
```

Raw email may be stored locally when necessary, but privacy-sensitive presentation should be considered carefully.

### File

```text
FileId
path
first_seen_commit
last_seen_commit
created_at
deleted_at
language
```

### FileChange

```text
commit_id
file_id
change_type
old_path
new_path
insertions
deletions
lines_changed
```

### Branch

```text
BranchId
name
tip_commit
created_at_if_known
last_seen
is_remote
is_default
```

### Tag

```text
TagId
name
target
tagger
timestamp
```

### ReflogEntry

```text
reference
old_oid
new_oid
actor
timestamp
message
```

## 2. Derived entities

### Ownership

```text
file_id
author_id
weighted_contribution
percentage
```

### Hotspot

```text
file_id
score
classification
change_frequency
recent_churn
bug_fix_frequency
ownership_concentration
complexity
```

### RepositoryHealth

```text
overall_score
classification
sub_scores:
  code_hotspots
  ownership_risk
  branch_hygiene
  change_volatility
  architecture_stability
  recovery_risk
```

Each sub-score carries the same evidence fields as any other derived metric.

### BranchAnalysis

```text
branch_id
ahead
behind
age
divergence
shared_files
estimated_merge_complexity
```

### ArchitectureSnapshot

```text
snapshot_ref
timestamp
nodes
edges
```

## 3. Identity normalization

Contributor identity is difficult because one developer may commit under:

```text
Abuzar
abuzar
Abuzar Khan
different@example.com
```

Identity normalization must be configurable.

Never silently merge identities based only on weak guesses.

Provide:

- raw identity
- normalized display identity
- optional user mapping configuration

## 4. Change types

At minimum:

```text
Added
Modified
Deleted
Renamed
Copied
TypeChanged
Unknown
```

## 5. Commit classifications

Classification is heuristic and must remain explicitly labeled:

```text
Feature
Fix
Refactor
Docs
Test
Chore
Revert
Merge
Unknown
```

Do not present heuristic classification as authoritative.

## 6. Time model

Use UTC internally.

Convert to local time only at presentation boundaries.

Store timestamps with enough precision for Git's actual commit timestamp semantics.
