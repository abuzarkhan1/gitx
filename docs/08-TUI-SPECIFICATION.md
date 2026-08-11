# TUI Specification

## 1. Design objective

The TUI should let a developer move from repository overview to historical evidence without leaving the terminal.

## 2. Main layout

```text
┌─────────────────────────────────────────────────────────────┐
│ GitX   Repository   Branch   State                           │
├──────────────┬──────────────────────────────────────────────┤
│ Navigation   │ Content                                      │
│              │                                              │
│ Overview     │                                              │
│ Timeline     │                                              │
│ Commits      │                                              │
│ Branches     │                                              │
│ Files        │                                              │
│ Contributors │                                              │
│ Hotspots     │                                              │
│ Ownership    │                                              │
│ Architecture │                                              │
│ Dependencies │                                              │
│ Risk         │                                              │
│ Health       │                                              │
│ Recovery     │                                              │
│ Search       │                                              │
└──────────────┴──────────────────────────────────────────────┘
```

## 3. Views

### Overview

Show:

- repository name/path
- current branch
- HEAD
- clean/dirty state
- repository age
- commit count
- contributor count
- branch count
- tag count
- language breakdown
- activity chart
- top hotspots
- recent commits
- health summary with sub-scores

### Timeline

Show chronological commits with:

- graph
- abbreviated OID
- author
- timestamp
- message
- changed-file count

### Commit view

Panels:

- commit metadata
- parent graph
- message
- changed files
- additions/deletions
- related commits
- affected areas

### File view

Show:

- current path
- lineage
- first/last change
- contributors
- churn
- commits
- rename events
- hotspot metrics

### Branch view

Show:

- branches
- ahead/behind
- age
- activity
- divergence
- shared files

### Contributors

Show:

- commits
- files
- contribution weight
- areas
- ownership concentration

### Hotspots

Sortable table:

```text
File
Score
Changes
Churn
Bug Fixes
Contributors
Ownership
```

### Health

Show the composite repository health score with its six sub-scores:

```text
Code Hotspots          HIGH     78/100
Ownership Risk         MEDIUM   64/100
Branch Hygiene         HIGH     81/100
Change Volatility      MEDIUM   71/100
Architecture Stability HIGH    83/100
Recovery Risk          HIGH     91/100
```

Selecting a sub-score must reveal the underlying evidence (files, branches, signals), never just a number.

### Architecture

Show structural graph or table depending on terminal dimensions.

### Recovery

Show:

- reflog entries
- unreachable commits
- dangling objects
- age
- reference paths

Recovery actions must clearly distinguish read-only inspection from operations that mutate the repository.

## 4. Navigation

Required:

```text
↑ / k      up
↓ / j      down
← / h      back
→ / l      open
Enter      select
Esc        close dialog
/          search
?          help
r          refresh
q          quit
```

Exact bindings may evolve, but Vim-style navigation should remain supported.

## 5. Responsive layout

Terminal width and height must be considered.

At small widths:

- hide secondary columns
- collapse navigation
- use stacked panels

Never allow important information to be silently truncated without an alternate inspection path.

## 6. Loading states

Long operations must show:

- operation name
- progress where measurable
- processed/total where available
- cancellation hint

## 7. Empty states

Example:

```text
No hotspot analysis available.

Run:
  gitx refresh
```

## 8. Errors

Errors should be actionable:

```text
Unable to read repository index.

Reason:
  Index schema is newer than this GitX build.

Suggested action:
  Upgrade GitX or rebuild the index.
```

## 9. Accessibility

- do not rely only on color
- use symbols/text labels for states
- maintain readable contrast
- support non-color terminals
- support keyboard-only operation
