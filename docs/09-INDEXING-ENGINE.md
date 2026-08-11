# Indexing Engine

## 1. Purpose

The index is the bridge between Git's raw object model and GitX's fast analytical queries.

## 2. Initial indexing

Pipeline:

```text
Discover repository
      ↓
Read refs
      ↓
Enumerate reachable commits
      ↓
Read commit metadata
      ↓
Read parent relationships
      ↓
Read trees/diffs
      ↓
Build file history
      ↓
Detect renames where supported
      ↓
Index branches/tags
      ↓
Index reflog
      ↓
Build search index
      ↓
Calculate base metrics
```

## 3. Incremental indexing

Before scanning, identify:

- current HEAD
- branch tips
- tag changes
- reflog changes
- index format version
- Git object changes

Then calculate the minimal new work.

### Example

```text
Previous HEAD: A
Current HEAD:  D

A → B → C → D
```

Only commits B/C/D and their affected derived data need processing if they are not already indexed.

## 4. Correctness rule

Never assume that only HEAD changed.

Branches can move independently.

Tags can move.

Reflog can change.

A force push can rewrite reachable history.

The indexer must detect ref changes and handle rewritten history safely.

## 5. Index invalidation

Cases requiring special handling:

- force-pushed branch
- rebased history
- deleted branch
- deleted tag
- rewritten refs
- repository garbage collection
- changed Git configuration affecting interpretation

## 6. Transaction model

Each logical indexing batch should be transactional.

Bad:

```text
write commit A
write commit B
crash
half-indexed state
```

Preferred:

```text
begin
  write batch
  update metadata
commit
```

On failure:

```text
rollback
```

## 7. Cancellation

Long indexing should be cancellable.

Cancellation must leave the index in a consistent state.

## 8. Progress

Track:

```text
objects discovered
commits processed
files processed
changes processed
search records indexed
```

Progress should be estimates when total work is unknown.

## 9. Rebuild

`gitx index rebuild` must:

1. create a fresh temporary index
2. populate it
3. validate it
4. atomically replace the old index

Never destroy the only working index before the replacement is valid.

## 10. Corruption recovery

If index integrity fails:

```text
detect
→ report
→ preserve diagnostic information
→ rebuild safely
```

The repository itself must remain untouched.
