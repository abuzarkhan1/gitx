# Product Requirements Document

## 1. Product

**Name:** GitX

**Category:** Local developer CLI / terminal repository intelligence tool

**One-line description:**

> GitX lets developers explore what happened in a Git repository, how the codebase evolved, where change and maintenance risk concentrate, who owns important areas, how branches and architecture changed, and what work can still be recovered.

## 2. Problem

Git provides excellent primitives, but understanding a mature repository usually requires combining many commands:

- `git log`
- `git show`
- `git diff`
- `git blame`
- `git branch`
- `git reflog`
- `git log --follow`
- `git shortlog`
- `git tag`
- dependency inspection
- filesystem inspection
- ad-hoc scripts

The problem is not lack of raw data. The problem is that the data is fragmented.

A developer may need to answer:

- When was this file introduced?
- Why has it changed so often?
- Which commits caused its current shape?
- Who has worked on it?
- Which files are the repository's biggest change hotspots?
- Which branches have diverged significantly?
- Which areas have concentrated ownership?
- How has the architecture changed?
- Which dependencies were introduced or removed?
- What deleted or unreachable work can still be recovered?
- What changed between two releases?

GitX combines these signals into one local, interactive, explainable system.

## 3. Goals

### Primary goals

1. Build a high-quality interactive TUI for repository exploration.
2. Provide a complete, indexed Git history model.
3. Provide file and commit archaeology, including rename lineage and line-level history.
4. Provide deterministic repository intelligence.
5. Provide hotspot and maintenance-risk analysis and a composite repository health score.
6. Provide ownership and contributor analysis.
7. Provide branch and merge intelligence.
8. Provide architecture/dependency evolution analysis.
9. Provide recovery intelligence for reflog and unreachable objects.
10. Provide fast full-text and structured search.
11. Provide JSON output for every major analytical command.
12. Maintain local-only operation by default.
13. Support large repositories through incremental indexing and caching.

### Secondary goals

- Cross-platform binary distribution.
- Excellent keyboard-driven UX.
- Strong test fixture system.
- Stable machine-readable output contracts.
- Clear explainability for every derived metric.

## 4. Non-goals

GitX will not become:

- a SaaS platform
- a web application
- a hosted Git provider
- a collaboration platform
- a GitHub/GitLab replacement
- an account/authentication system
- a cloud repository
- an AI assistant
- an AI chat interface
- a project-management application
- a CI/CD platform
- a remote code-hosting service

Remote Git operations may be supported only where necessary to inspect normal Git repository state; GitX is not responsible for hosting or managing remote services.

## 5. Target users

### Primary

- Software developers
- Maintainers
- Open-source contributors
- Engineers joining unfamiliar repositories
- Developers debugging historical regressions
- Developers investigating risky areas before modifying code

### Secondary

- Technical leads
- Code reviewers
- Repository maintainers
- Engineers performing migrations or refactors

## 6. Core use cases

### UC-01: Understand a repository

Developer opens:

```bash
gitx
```

and immediately sees repository health, activity, branches, contributors, and important areas.

### UC-02: Investigate a commit

```bash
gitx commit <hash>
```

shows metadata, parents, changed files, diff statistics, affected areas, related commits, and impact metrics.

### UC-03: Investigate a file

```bash
gitx history path/to/file
```

shows creation, rename, modification, contributors, churn, change frequency, and historical evolution.

### UC-04: Find hotspots

```bash
gitx hotspots
```

ranks files by deterministic maintenance-risk signals.

### UC-05: Understand ownership

```bash
gitx ownership
```

shows contribution distribution and concentrated ownership.

### UC-06: Investigate branches

```bash
gitx branches
```

shows divergence, age, ahead/behind state, shared files, and potential merge complexity.

### UC-07: Investigate architecture evolution

```bash
gitx architecture
```

compares repository structure and dependency relationships over time.

### UC-08: Recover lost work

```bash
gitx recovery
```

finds reflog entries, unreachable commits, dangling objects, and deleted branch references.

### UC-09: Search historical data

```bash
gitx search "workspace"
```

searches indexed commit messages, authors, paths, symbols, branches, tags, and other supported entities.

## 7. Product principles

### Explainability

Every derived metric must expose its inputs.

Bad:

```text
Risk: 87
```

Good:

```text
Risk: 87/100

Change frequency     92
Recent churn         88
Bug-fix frequency    81
Ownership concentration 91
Complexity           74
```

### Determinism

For the same repository snapshot and configuration, the same analysis must produce the same result.

### Local privacy

Repository contents and history must not leave the machine as part of normal GitX operation.

### Progressive disclosure

The TUI should show useful summaries first and detailed evidence only when requested.

## 8. Success criteria

The MVP is successful when:

- a developer can open a repository and understand its current state quickly;
- historical investigation requires substantially fewer manual Git commands;
- every analysis result can be traced to repository evidence;
- indexing is incremental;
- the TUI remains responsive on representative medium/large repositories;
- JSON output is stable enough for scripts;
- the tool works without an internet connection;
- tests cover Git history edge cases.

## 9. MVP definition

MVP must include:

- repository discovery
- Git history indexing
- commits and parents
- branches and tags
- authors
- file history
- rename tracking
- diffs/statistics
- interactive TUI
- timeline
- commit explorer
- file explorer
- hotspot analysis
- ownership analysis
- repository health overview
- branch intelligence
- search
- reflog/recovery analysis
- SQLite cache/index
- JSON output
- configuration
- cross-platform build
- tests and fixtures

Architecture/dependency evolution may begin in MVP with directory/file-level analysis and expand to language-aware structural analysis later.
